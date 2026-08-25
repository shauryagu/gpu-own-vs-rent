//! Discrete NPV identity. No I/O.

use crate::money::{Usd, UsdPerGpuHour};
use crate::qty::{DiscountRate, GpuHour, Utilization, Years};
use crate::spot::ObservedSpot;
use crate::theta::{Theta, ThetaExResidual, HOURS_PER_YEAR};
use rust_decimal::Decimal;
use thiserror::Error;

/// Failed to evaluate the identity or the energy product.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum IdentityError {
    /// A physics product was NaN or infinite at the `f64` → `Decimal` step.
    #[error("physics quantity is not finite")]
    NonFinitePhysics,
    /// \( h A = 0 \). Unreachable via public constructors (\( T \ge 1 \), \( r \ge 0 \)
    /// \(\Rightarrow A \ge 1\) or \( A > 0 \); \( u > 0 \)). Fail-closed belt.
    #[error("utilized hours times annuity factor is zero")]
    ZeroAnnuity,
}

/// \( (1+r)^{T} \). \( r = 0 \) is 1, so we never raise zero.
fn one_plus_r_pow_t(rate: DiscountRate, life: Years) -> Decimal {
    let r = rate.amount();
    if r.is_zero() {
        return Decimal::ONE;
    }
    let one_plus_r = Decimal::ONE + r;
    let mut grow = Decimal::ONE;
    for _ in 0..life.get() {
        grow *= one_plus_r;
    }
    grow
}

/// \( A(r,T) = T \) if \( r = 0 \), else \( (1 - (1+r)^{-T}) / r \).
fn annuity_factor(rate: DiscountRate, life: Years) -> Decimal {
    let r = rate.amount();
    if r.is_zero() {
        return Decimal::from(life.get());
    }
    let grow = one_plus_r_pow_t(rate, life);
    (Decimal::ONE - Decimal::ONE / grow) / r
}

fn hours_times_annuity(
    hours: GpuHour,
    rate: DiscountRate,
    life: Years,
) -> Result<Decimal, IdentityError> {
    let denom = hours.amount() * annuity_factor(rate, life);
    // Fail-closed belt: public constructors make A = 0 unreachable (T ≥ 1, r ≥ 0).
    if denom.is_zero() {
        return Err(IdentityError::ZeroAnnuity);
    }
    Ok(denom)
}

/// \( h = u H \).
fn utilized_hours(u: Utilization) -> GpuHour {
    u * HOURS_PER_YEAR
}

impl ThetaExResidual {
    /// \( h = u H \).
    pub fn utilized_hours_per_year(&self) -> Result<GpuHour, IdentityError> {
        Ok(utilized_hours(self.utilization))
    }
}

/// `TryFrom<Decimal>` is infallible today. One wrap so a later check is not a
/// library `expect` on every identity path, and is not `NonFinitePhysics`.
pub(crate) fn usd(d: Decimal) -> Usd {
    Usd::try_from(d).expect("TryFrom<Decimal> for Usd is infallible")
}

pub(crate) fn rate(d: Decimal) -> UsdPerGpuHour {
    UsdPerGpuHour::try_from(d).expect("TryFrom<Decimal> for UsdPerGpuHour is infallible")
}

/// Capital-recovery rent \( F_{\mathrm{capital}} = P / (h A) \). Not a second name for \( F(\theta) \).
pub fn capital_rent(ex: &ThetaExResidual) -> Result<UsdPerGpuHour, IdentityError> {
    let hours = utilized_hours(ex.utilization);
    let denom = hours_times_annuity(hours, ex.discount, ex.life)?;
    Ok(ex.purchase / GpuHour::new(denom))
}

/// User cost of capital / fair rent \( F(\theta) \). Not Bandi & Su \( F_t(T) \).
///
/// Cash-and-carry fails for a GPU-hour (Bandi & Su §5.1). This is the discrete
/// NPV identity, not a futures price and not a carry engine.
pub fn fair_rent(theta: &Theta) -> Result<UsdPerGpuHour, IdentityError> {
    let hours = utilized_hours(theta.utilization);
    let denom = hours_times_annuity(hours, theta.discount, theta.life)?;
    let grow = one_plus_r_pow_t(theta.discount, theta.life);
    let discounted_salvage = theta.salvage.amount() / grow;
    let capital = (theta.purchase.amount() - discounted_salvage) / denom;
    Ok(rate(theta.energy.amount() + capital))
}

/// Implied salvage \( R^{\star}(S) \). USD at year \( T \). Falls with \( S \).
///
/// Negative values are valid. This is not leftover and not clamped.
pub fn implied_salvage(obs: ObservedSpot, ex: &ThetaExResidual) -> Result<Usd, IdentityError> {
    let hours = utilized_hours(ex.utilization);
    let denom = hours_times_annuity(hours, ex.discount, ex.life)?;
    let grow = one_plus_r_pow_t(ex.discount, ex.life);
    let inner = ex.purchase.amount() - (obs.price.amount() - ex.energy.amount()) * denom;
    Ok(usd(grow * inner))
}

/// Leftover \( L(S) = S - F_{\mathrm{capital}} - e \). USD per GPU-hour. Rises with \( S \).
pub fn leftover(obs: ObservedSpot, ex: &ThetaExResidual) -> Result<UsdPerGpuHour, IdentityError> {
    let capital = capital_rent(ex)?;
    Ok(rate(
        obs.price.amount() - capital.amount() - ex.energy.amount(),
    ))
}

#[cfg(test)]
mod tests {
    use super::{capital_rent, fair_rent, implied_salvage, leftover};
    use crate::theta::{Theta, ThetaExResidual, HOURS_PER_YEAR};
    use crate::{
        DiscountRate, GpuModel, ObservedSpot, SpotSeries, Usd, UsdPerGpuHour, Utilization, ValidOn,
        Years,
    };
    use rust_decimal::Decimal;
    use time::{Date, Month};

    fn dec(s: &str) -> Decimal {
        Decimal::from_str_exact(s).expect("fixture decimal")
    }

    /// Teaching numbers, not defaults: P = 10000, T = 2, u = 0.5, r = 0, e = 0.
    fn ex_zero_discount() -> ThetaExResidual {
        ThetaExResidual {
            purchase: Usd::from_cents(1_000_000),
            life: Years::try_new(2).unwrap(),
            utilization: Utilization::try_new(dec("0.5")).unwrap(),
            energy: UsdPerGpuHour::from_cents(0),
            discount: DiscountRate::try_new(Decimal::ZERO).unwrap(),
        }
    }

    fn spot(price: UsdPerGpuHour) -> ObservedSpot {
        ObservedSpot {
            gpu: GpuModel::H100Sxm,
            series: SpotSeries::OcpiDailyIndex,
            valid_on: ValidOn::new(Date::from_calendar_date(2026, Month::August, 21).unwrap()),
            price,
        }
    }

    #[test]
    fn capital_rent_at_zero_discount_uses_annuity_equal_to_life() {
        let ex = ex_zero_discount();
        let f = capital_rent(&ex).expect("r=0 uses A=T");
        // h = 0.5 · 8760 = 4380; A = T = 2; hA = 8760 = H.
        let expected = ex.purchase / HOURS_PER_YEAR;
        assert_eq!(f, expected);
    }

    #[test]
    fn leftover_is_spot_minus_capital_rent_minus_energy() {
        let mut ex = ex_zero_discount();
        ex.energy = UsdPerGpuHour::try_from(dec("0.05")).expect("e");
        let s = UsdPerGpuHour::try_from(dec("3.00")).expect("S");
        let l = leftover(spot(s), &ex).expect("r=0 uses A=T");
        let f = capital_rent(&ex).expect("r=0 uses A=T");
        assert_eq!(l.amount(), s.amount() - f.amount() - ex.energy.amount());
    }

    fn theta_from_ex(ex: ThetaExResidual, salvage: Usd) -> Theta {
        Theta {
            purchase: ex.purchase,
            life: ex.life,
            utilization: ex.utilization,
            salvage,
            energy: ex.energy,
            discount: ex.discount,
        }
    }

    fn relatively_close(a: Decimal, b: Decimal) -> bool {
        let scale = a.abs().max(b.abs());
        if scale.is_zero() {
            return a.is_zero() && b.is_zero();
        }
        (a - b).abs() / scale < dec("0.00000001")
    }

    #[test]
    fn fair_rent_is_usd_per_gpu_hour() {
        let theta = theta_from_ex(ex_zero_discount(), Usd::from_cents(0));
        let f: UsdPerGpuHour = fair_rent(&theta).expect("r=0");
        // r=0, R=0 ⇒ F(θ) = e + P/(hA) = F_capital.
        assert_eq!(f, capital_rent(&ex_zero_discount()).expect("r=0"));
    }

    #[test]
    fn implied_salvage_negative_equals_the_formula_not_a_floor() {
        let ex = ex_zero_discount();
        let s = UsdPerGpuHour::try_from(dec("3.00")).expect("S");
        let r_star = implied_salvage(spot(s), &ex).expect("r=0");
        // r=0: R* = P − (S−e) h T. hT = 8760; 3·8760 = 26280; 10000 − 26280 = −16280.
        let hours = ex.utilized_hours_per_year().expect("u>0");
        let expected = ex.purchase.amount()
            - (s.amount() - ex.energy.amount()) * hours.amount() * Decimal::from(ex.life.get());
        assert_eq!(r_star.amount(), expected);
        assert!(r_star.amount() < Decimal::ZERO);
        assert_ne!(r_star.amount(), expected.max(Decimal::ZERO));
    }

    #[test]
    fn fair_rent_of_implied_salvage_round_trips_spot() {
        let ex = ex_zero_discount();
        let s = UsdPerGpuHour::try_from(dec("3.00")).expect("S");
        let r_star = implied_salvage(spot(s), &ex).expect("R*");
        let f = fair_rent(&theta_from_ex(ex, r_star)).expect("F(R*)");
        assert!(
            relatively_close(f.amount(), s.amount()),
            "F(R*(S)) = {f:?}, S = {s:?}"
        );
    }

    #[test]
    fn implied_salvage_of_fair_rent_round_trips_salvage() {
        let ex = ex_zero_discount();
        let salvage = Usd::from_cents(-50_000);
        let f = fair_rent(&theta_from_ex(ex, salvage)).expect("F(θ)");
        let r_star = implied_salvage(spot(f), &ex).expect("R*(F)");
        assert!(
            relatively_close(r_star.amount(), salvage.amount()),
            "R*(F(θ)) = {r_star:?}, R = {salvage:?}"
        );
    }

    #[test]
    fn zero_utilization_or_life_is_err_at_construction() {
        // Invalid u and T are unrepresentable. ZeroAnnuity is a belt, not proven here.
        assert!(Utilization::try_new(Decimal::ZERO).is_err());
        assert!(Years::try_new(0).is_err());
    }
}
