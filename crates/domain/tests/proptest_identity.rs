//! Gate 1 identity properties.
//!
//! P3 is not a universal theorem (∂F/∂u and ∂F/∂T are not always negative).
//! It is omitted. P1a and P1b are both here; leftover and salvage stay distinct.

use domain::{
    energy_per_gpu_hour, fair_rent, from_h100e_as_h100, implied_salvage, leftover, to_h100e,
    DiscountRate, GpuHour, GpuModel, Kilowatt, ObservedSpot, Pue, SpotSeries, Theta,
    ThetaExResidual, Usd, UsdPerGpuHour, UsdPerKwh, Utilization, ValidOn, Years,
};
use proptest::prelude::*;
use rust_decimal::Decimal;
use time::{Date, Month};

fn relatively_close(a: Decimal, b: Decimal) -> bool {
    let scale = a.abs().max(b.abs());
    if scale.is_zero() {
        return a.is_zero() && b.is_zero();
    }
    (a - b).abs() / scale < Decimal::new(1, 8)
}

fn spot(price: UsdPerGpuHour) -> ObservedSpot {
    ObservedSpot {
        gpu: GpuModel::H100Sxm,
        series: SpotSeries::OcpiDailyIndex,
        valid_on: ValidOn::new(Date::from_calendar_date(2026, Month::August, 21).unwrap()),
        price,
    }
}

fn utilization(parts: u32) -> Utilization {
    Utilization::try_new(Decimal::new(i64::from(parts), 4)).expect("u in (0, 1]")
}

fn discount(parts: u32) -> DiscountRate {
    DiscountRate::try_new(Decimal::new(i64::from(parts), 4)).expect("r >= 0")
}

fn ex(
    purchase_cents: i64,
    life: u32,
    util_parts: u32,
    energy_cents: i64,
    discount_parts: u32,
) -> ThetaExResidual {
    ThetaExResidual {
        purchase: Usd::from_cents(purchase_cents),
        life: Years::try_new(life).expect("T >= 1"),
        utilization: utilization(util_parts),
        energy: UsdPerGpuHour::from_cents(energy_cents),
        discount: discount(discount_parts),
    }
}

fn theta_from(ex: ThetaExResidual, salvage: Usd) -> Theta {
    Theta {
        purchase: ex.purchase,
        life: ex.life,
        utilization: ex.utilization,
        salvage,
        energy: ex.energy,
        discount: ex.discount,
    }
}

fn rate_from_cents(cents: i64) -> UsdPerGpuHour {
    UsdPerGpuHour::from_cents(cents)
}

proptest! {
    #[test]
    fn p1a_leftover_rises_with_spot(
        purchase_cents in 1i64..=10_000_000,
        life in 1u32..=15,
        util_parts in 1u32..=10_000,
        discount_parts in 0u32..=5_000,
        energy_cents in 0i64..=1_000_000,
        s1_extra_cents in 1i64..=1_000_000,
        delta_cents in 1i64..=1_000_000,
    ) {
        let ex = ex(purchase_cents, life, util_parts, energy_cents, discount_parts);
        let s1 = rate_from_cents(energy_cents + s1_extra_cents);
        let s2 = rate_from_cents(energy_cents + s1_extra_cents + delta_cents);
        let l1 = leftover(spot(s1), &ex).expect("hA > 0");
        let l2 = leftover(spot(s2), &ex).expect("hA > 0");
        prop_assert!(l2.amount() > l1.amount());
    }

    #[test]
    fn p1b_implied_salvage_falls_with_spot(
        purchase_cents in 1i64..=10_000_000,
        life in 1u32..=15,
        util_parts in 1u32..=10_000,
        discount_parts in 0u32..=5_000,
        energy_cents in 0i64..=1_000_000,
        s1_extra_cents in 1i64..=1_000_000,
        delta_cents in 1i64..=1_000_000,
    ) {
        let ex = ex(purchase_cents, life, util_parts, energy_cents, discount_parts);
        let s1 = rate_from_cents(energy_cents + s1_extra_cents);
        let s2 = rate_from_cents(energy_cents + s1_extra_cents + delta_cents);
        let r1 = implied_salvage(spot(s1), &ex).expect("hA > 0");
        let r2 = implied_salvage(spot(s2), &ex).expect("hA > 0");
        prop_assert!(r2.amount() < r1.amount());
    }

    #[test]
    fn p2_fair_rent_falls_as_salvage_rises(
        purchase_cents in 1i64..=10_000_000,
        life in 1u32..=15,
        util_parts in 1u32..=10_000,
        discount_parts in 0u32..=5_000,
        energy_cents in 0i64..=1_000_000,
        salvage_cents in -10_000_000i64..=10_000_000,
        delta_cents in 1i64..=1_000_000,
    ) {
        prop_assume!(salvage_cents.checked_add(delta_cents).is_some());
        let ex = ex(purchase_cents, life, util_parts, energy_cents, discount_parts);
        let r1 = Usd::from_cents(salvage_cents);
        let r2 = Usd::from_cents(salvage_cents + delta_cents);
        let f1 = fair_rent(&theta_from(ex, r1)).expect("hA > 0");
        let f2 = fair_rent(&theta_from(ex, r2)).expect("hA > 0");
        prop_assert!(f2.amount() < f1.amount());
    }

    #[test]
    fn p4_fair_rent_and_implied_salvage_round_trip(
        purchase_cents in 1i64..=10_000_000,
        life in 1u32..=15,
        util_parts in 1u32..=10_000,
        discount_parts in 0u32..=5_000,
        energy_cents in 0i64..=1_000_000,
        salvage_cents in -10_000_000i64..=10_000_000,
        extra_cents in 1i64..=1_000_000,
    ) {
        let ex = ex(purchase_cents, life, util_parts, energy_cents, discount_parts);
        let salvage = Usd::from_cents(salvage_cents);
        let s = rate_from_cents(energy_cents + extra_cents);

        let r_star = implied_salvage(spot(s), &ex).expect("hA > 0");
        let f_of_r_star = fair_rent(&theta_from(ex, r_star)).expect("hA > 0");
        prop_assert!(relatively_close(f_of_r_star.amount(), s.amount()));

        let f = fair_rent(&theta_from(ex, salvage)).expect("hA > 0");
        let r_of_f = implied_salvage(spot(f), &ex).expect("hA > 0");
        prop_assert!(relatively_close(r_of_f.amount(), salvage.amount()));
    }

    #[test]
    fn p5_h100_sxm_round_trips_as_h100e(hours in 1i64..=1_000_000) {
        let h = GpuHour::from(hours);
        let h100e = to_h100e(h, GpuModel::H100Sxm).expect("H100 SXM is identity");
        prop_assert_eq!(from_h100e_as_h100(h100e), h);
    }

    #[test]
    fn p6_energy_is_the_product_including_zero_price(
        tdp_mw in 100u32..=2_000,
        price_cents in 0i64..=10_000,
        pue_milli in 1_000u32..=2_000,
    ) {
        let tdp = Kilowatt::try_new(f64::from(tdp_mw) / 1_000.0).expect("finite kW");
        let price = UsdPerKwh::from_cents(price_cents);
        let pue = Pue::try_new(f64::from(pue_milli) / 1_000.0).expect("finite PUE");
        let e = energy_per_gpu_hour(tdp, price, pue).expect("finite physics");
        let physics = Decimal::from_f64_retain(tdp.get() * 1.0 * pue.get()).expect("finite");
        prop_assert_eq!(e.amount(), physics * price.amount());
    }
}
