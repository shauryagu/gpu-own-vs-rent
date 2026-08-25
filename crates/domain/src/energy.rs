//! Energy cost per utilized GPU-hour.

use crate::identity::{rate, IdentityError};
use crate::money::{UsdPerGpuHour, UsdPerKwh};
use crate::qty::{Hours, Kilowatt, Pue};
use rust_decimal::Decimal;

/// TDP[kW] · 1[h] · π[USD/kWh] · PUE, as USD per GPU-hour.
///
/// The product is evaluated even when π is zero.
pub fn energy_per_gpu_hour(
    tdp: Kilowatt,
    price: UsdPerKwh,
    pue: Pue,
) -> Result<UsdPerGpuHour, IdentityError> {
    let hour = Hours::try_new(1.0).expect("1 h is finite");
    let kwh = tdp * hour;
    let physics =
        Decimal::from_f64_retain(kwh * pue.get()).ok_or(IdentityError::NonFinitePhysics)?;
    Ok(rate(physics * price.amount()))
}

#[cfg(test)]
mod tests {
    use super::energy_per_gpu_hour;
    use crate::{IdentityError, Kilowatt, Pue, UsdPerKwh};
    use rust_decimal::Decimal;

    #[test]
    fn energy_at_zero_price_is_the_product_not_a_hardcoded_zero() {
        let tdp = Kilowatt::try_new(0.7).unwrap();
        let price = UsdPerKwh::from_cents(0);
        let pue = Pue::try_new(1.0).unwrap();
        let e = energy_per_gpu_hour(tdp, price, pue).expect("finite physics");
        assert_eq!(e.amount(), Decimal::ZERO);
    }

    #[test]
    fn energy_at_ten_cents_per_kwh_is_the_product() {
        let tdp = Kilowatt::try_new(0.7).unwrap();
        let price = UsdPerKwh::from_cents(10);
        let pue = Pue::try_new(1.0).unwrap();
        let e = energy_per_gpu_hour(tdp, price, pue).expect("finite physics");
        // 0.7 f64 is not the decimal 7/10. The paper product is 0.07;
        // from_f64_retain keeps the binary value, then money is Decimal.
        let physics = Decimal::from_f64_retain(0.7).expect("0.7 is finite");
        assert_eq!(e.amount(), physics * price.amount());
        assert!(e.amount() > Decimal::ZERO);
    }

    #[test]
    fn energy_rejects_non_finite_physics_product() {
        let tdp = Kilowatt::try_new(f64::MAX).unwrap();
        let price = UsdPerKwh::from_cents(10);
        let pue = Pue::try_new(f64::MAX).unwrap();
        assert!(matches!(
            energy_per_gpu_hour(tdp, price, pue),
            Err(IdentityError::NonFinitePhysics)
        ));
    }
}
