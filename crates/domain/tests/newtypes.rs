//! Public-API tests for Gate 1 domain newtypes (PR 3).

use domain::{
    from_h100e_as_h100, to_h100e, ConvertError, DiscountRate, GpuHour, GpuModel, Hours, Kilowatt,
    ObservedSpot, Pue, SpotSeries, Usd, UsdPerGpuHour, Utilization, ValidOn, Years,
};
use rust_decimal::Decimal;
use time::{Date, Month};

fn dec(s: &str) -> Decimal {
    Decimal::from_str_exact(s).expect("fixture decimal")
}

#[test]
fn rate_times_hours_is_usd() {
    let rate = UsdPerGpuHour::try_from(dec("2.50")).expect("2.50");
    let hours = GpuHour::new(dec("10"));
    assert_eq!(rate * hours, Usd::from_cents(2500));
}

#[test]
fn usd_div_hours_is_rate() {
    let total = Usd::from_cents(2500);
    let hours = GpuHour::new(dec("10"));
    assert_eq!(
        total / hours,
        UsdPerGpuHour::try_from(dec("2.50")).expect("2.50")
    );
}

#[test]
fn utilization_scales_gpu_hours() {
    let u = Utilization::try_new(dec("0.5")).expect("u=0.5");
    let civil = GpuHour::new(dec("8760"));
    assert_eq!(u * civil, GpuHour::new(dec("4380")));
}

#[test]
fn usd_plus_usd_is_usd() {
    assert_eq!(
        Usd::from_cents(100) + Usd::from_cents(50),
        Usd::from_cents(150)
    );
}

#[test]
fn kilowatt_times_hours_is_kwh() {
    let tdp = Kilowatt::try_new(0.7).expect("0.7 kW");
    let hour = Hours::try_new(1.0).expect("1 h");
    assert!((tdp * hour - 0.7).abs() < 1e-12);
}

#[test]
fn utilization_zero_is_rejected() {
    assert!(Utilization::try_new(Decimal::ZERO).is_err());
}

#[test]
fn utilization_above_one_is_rejected() {
    assert!(Utilization::try_new(dec("1.01")).is_err());
}

#[test]
fn utilization_one_is_allowed() {
    assert!(Utilization::try_new(Decimal::ONE).is_ok());
}

#[test]
fn usd_serde_is_decimal_string_not_json_number() {
    let usd = Usd::from_cents(2500);
    let json = serde_json::to_string(&usd).expect("serialize");
    assert_eq!(json, "\"25.00\"");

    let parsed: Usd = serde_json::from_str("\"25.00\"").expect("string form");
    assert_eq!(parsed, usd);

    assert!(
        serde_json::from_str::<Usd>("25.0").is_err(),
        "JSON number must be a schema error"
    );
}

#[test]
fn money_serde_uses_display_scale_not_source_token() {
    // Domain serde is the printed quantity, not the OCPI ingest wire.
    // Ingest must keep index_value as token text; invert JSON owns round_dp(12).
    let rate = UsdPerGpuHour::try_from(dec("2.879583333333333")).expect("S");
    assert_eq!(
        serde_json::to_string(&rate).expect("serialize"),
        "\"2.8796\""
    );

    let total = Usd::try_from(dec("25.006")).expect("usd");
    assert_eq!(
        serde_json::to_string(&total).expect("serialize"),
        "\"25.01\""
    );
}

#[test]
fn h100_sxm_round_trips_as_h100e_identity() {
    let hours = GpuHour::new(dec("10"));
    let h100e = to_h100e(hours, GpuModel::H100Sxm).expect("H100 SXM is identity");
    assert_eq!(from_h100e_as_h100(h100e), hours);
}

#[test]
fn other_gpus_are_not_in_mvp_conversion() {
    let hours = GpuHour::new(dec("1"));
    for gpu in [
        GpuModel::H200,
        GpuModel::B200,
        GpuModel::A100Sxm4,
        GpuModel::Rtx5090,
    ] {
        assert!(matches!(
            to_h100e(hours, gpu),
            Err(ConvertError::NotInMvp(got)) if got == gpu
        ));
    }
}

#[test]
fn observed_spot_series_is_only_ocpi_daily_index() {
    // Exhaustive match: OcpiCurrent and OcpiDailyHistory are absent on purpose.
    // Invert S cannot be built from a "current" series because the variant
    // does not exist.
    let series = SpotSeries::OcpiDailyIndex;
    match series {
        SpotSeries::OcpiDailyIndex => {}
    }

    let spot = ObservedSpot {
        gpu: GpuModel::H100Sxm,
        series: SpotSeries::OcpiDailyIndex,
        valid_on: ValidOn::new(Date::from_calendar_date(2026, Month::August, 21).unwrap()),
        price: UsdPerGpuHour::try_from(dec("2.879583333333333")).expect("S"),
    };
    assert!(matches!(spot.series, SpotSeries::OcpiDailyIndex));
}

#[test]
fn years_zero_is_rejected() {
    assert!(Years::try_new(0).is_err());
    assert!(Years::try_new(1).is_ok());
}

#[test]
fn discount_rate_rejects_negative() {
    assert!(DiscountRate::try_new(dec("-0.01")).is_err());
    assert!(DiscountRate::try_new(Decimal::ZERO).is_ok());
}

#[test]
fn physics_constructors_reject_non_finite() {
    assert!(Kilowatt::try_new(f64::NAN).is_err());
    assert!(Kilowatt::try_new(f64::INFINITY).is_err());
    assert!(Hours::try_new(f64::NEG_INFINITY).is_err());
    assert!(Pue::try_new(f64::NAN).is_err());
}
