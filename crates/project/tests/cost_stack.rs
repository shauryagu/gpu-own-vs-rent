use domain::{
    capital_rent, DiscountRate, GpuModel, Kilowatt, ObservedSpot, Pue, SpotSeries, ThetaExResidual,
    Usd, UsdPerGpuHour, UsdPerKwh, Utilization, ValidOn, Years,
};
use project::{default_pue_grid, CostStack, NamedEnergy};
use rust_decimal::Decimal;
use time::{Date, Month};

fn dec(s: &str) -> Decimal {
    Decimal::from_str_exact(s).expect("exact")
}

fn teaching_ex() -> ThetaExResidual {
    ThetaExResidual {
        purchase: Usd::from_cents(2_500_000),
        life: Years::try_new(5).unwrap(),
        utilization: Utilization::try_new(dec("0.60")).unwrap(),
        energy: UsdPerGpuHour::from_cents(0),
        discount: DiscountRate::try_new(dec("0.10")).unwrap(),
    }
}

fn teaching_spot() -> ObservedSpot {
    ObservedSpot {
        gpu: GpuModel::H100Sxm,
        series: SpotSeries::OcpiDailyIndex,
        valid_on: ValidOn::new(Date::from_calendar_date(2026, Month::August, 21).unwrap()),
        price: UsdPerGpuHour::try_from(dec("2.879583333333333")).unwrap(),
    }
}

fn named_pi(cents: i64) -> NamedEnergy {
    NamedEnergy {
        usd_per_kwh: UsdPerKwh::from_cents(cents),
        lmp_usd_per_mwh: Some("49.24".to_string()),
        source: "PJM RTO 2026-09-10".to_string(),
    }
}

#[test]
fn each_row_spot_equals_capital_plus_energy_plus_leftover() {
    let tdp = Kilowatt::try_new(0.7).unwrap();
    let pues = [Pue::try_new(1.0).unwrap(), Pue::try_new(1.5).unwrap()];
    let stack = CostStack::compute(teaching_spot(), teaching_ex(), tdp, named_pi(10), &pues)
        .expect("stack");
    assert_eq!(stack.rows.len(), 2);
    for row in &stack.rows {
        let sum = row.capital.amount() + row.energy.amount() + row.leftover.amount();
        assert_eq!(sum, stack.spot.amount());
    }
}

#[test]
fn higher_pue_scales_energy_not_capital_and_cuts_leftover() {
    let tdp = Kilowatt::try_new(0.7).unwrap();
    let pues = [Pue::try_new(1.0).unwrap(), Pue::try_new(1.5).unwrap()];
    let stack = CostStack::compute(teaching_spot(), teaching_ex(), tdp, named_pi(10), &pues)
        .expect("stack");
    let a = &stack.rows[0];
    let b = &stack.rows[1];
    assert_eq!(a.pue.get(), 1.0);
    assert_eq!(b.pue.get(), 1.5);
    assert_eq!(a.capital, b.capital);
    assert_eq!(a.capital, capital_rent(&teaching_ex()).unwrap());
    assert!(b.energy.amount() > a.energy.amount());
    assert!(b.leftover.amount() < a.leftover.amount());
    assert!(b.energy_share > a.energy_share);
}

#[test]
fn zero_pi_still_evaluates_energy_product() {
    let tdp = Kilowatt::try_new(0.7).unwrap();
    let pue = Pue::try_new(1.0).unwrap();
    let stack = CostStack::compute(teaching_spot(), teaching_ex(), tdp, named_pi(0), &[pue])
        .expect("π=0 is a product, not a skip");
    assert_eq!(stack.rows[0].energy.amount(), Decimal::ZERO);
    let capital = capital_rent(&teaching_ex()).unwrap();
    assert_eq!(
        stack.rows[0].leftover.amount(),
        stack.spot.amount() - capital.amount()
    );
}

#[test]
fn cost_stack_source_has_no_salvage_or_implied_residual() {
    let src = include_str!("../src/cost_stack.rs");
    for needle in ["implied_salvage", "implied residual", "salvage"] {
        assert!(!src.contains(needle), "CostStack must not mention {needle}");
    }
}

#[test]
fn default_pue_grid_is_one_point_two_one_point_five_in_order() {
    let grid = default_pue_grid();
    assert_eq!(grid.map(|p| p.get()), [1.0, 1.2, 1.5]);
}

#[test]
fn default_grid_compute_three_rows_capital_equal_leftover_falls() {
    let tdp = Kilowatt::try_new(0.7).unwrap();
    let grid = default_pue_grid();
    let stack = CostStack::compute(teaching_spot(), teaching_ex(), tdp, named_pi(10), &grid)
        .expect("stack");
    assert_eq!(stack.rows.len(), 3);
    assert_eq!(stack.rows[0].pue.get(), 1.0);
    assert_eq!(stack.rows[1].pue.get(), 1.2);
    assert_eq!(stack.rows[2].pue.get(), 1.5);
    assert_eq!(stack.rows[0].capital, stack.rows[1].capital);
    assert_eq!(stack.rows[1].capital, stack.rows[2].capital);
    assert!(stack.rows[0].leftover.amount() > stack.rows[1].leftover.amount());
    assert!(stack.rows[1].leftover.amount() > stack.rows[2].leftover.amount());
}

#[test]
fn single_pue_slice_still_yields_one_row() {
    let tdp = Kilowatt::try_new(0.7).unwrap();
    let pues = [Pue::try_new(1.0).unwrap()];
    let stack = CostStack::compute(teaching_spot(), teaching_ex(), tdp, named_pi(10), &pues)
        .expect("stack");
    assert_eq!(stack.rows.len(), 1);
    assert_eq!(stack.rows[0].pue.get(), 1.0);
}
