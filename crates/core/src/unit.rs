//! 单位换算模块:纯数学,无外部依赖
//!
//! 支持 FreeConvert 10 类常用单位:长度/面积/体积/质量/温度/时间/速度/数据/能量/频率。
//! 每类定义基准单位与换算系数(温度为非线性,单独处理)。

use crate::{ToolError, ToolResult};

/// 单位分类
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitCategory {
    Length,
    Area,
    Volume,
    Mass,
    Temperature,
    Time,
    Speed,
    Data,
    Energy,
    Frequency,
}

impl UnitCategory {
    /// 类别名称(中文)
    pub fn label(&self) -> &'static str {
        match self {
            UnitCategory::Length => "长度",
            UnitCategory::Area => "面积",
            UnitCategory::Volume => "体积",
            UnitCategory::Mass => "质量",
            UnitCategory::Temperature => "温度",
            UnitCategory::Time => "时间",
            UnitCategory::Speed => "速度",
            UnitCategory::Data => "数据",
            UnitCategory::Energy => "能量",
            UnitCategory::Frequency => "频率",
        }
    }
}

/// 单位换算:把 `value` 单位的值从 `from` 转为 `to`
///
/// 同类单位转换;跨类或未知单位返回 Err。温度为非线性(摄氏/华氏/开尔文)特殊处理。
pub fn unit_convert(value: f64, from: &str, to: &str) -> ToolResult<f64> {
    let (cat, from_factor, from_offset) = resolve_unit(from)?;
    let (_cat2, to_factor, to_offset) = resolve_unit(to)?;
    if resolve_unit(to)?.0 != cat {
        return Err(ToolError::InvalidInput(format!(
            "单位跨类: {from}({}) 与 {to}({})",
            cat.label(),
            resolve_unit(to)?.0.label()
        )));
    }
    // 转为基准值: value * from_factor + from_offset(温度偏移)
    let base = value * from_factor + from_offset;
    // 基准转目标:(base - to_offset) / to_factor
    Ok((base - to_offset) / to_factor)
}

/// 解析单位:返回 (类别, 转基准系数, 转基准偏移)
///
/// 线性单位 offset=0;温度以摄氏为基准:C→C 系数1/0偏移,F→C 系数5/9/偏移-32*5/9,K→C 系数1/偏移-273.15。
fn resolve_unit(unit: &str) -> ToolResult<(UnitCategory, f64, f64)> {
    let u = unit.to_lowercase();
    // 长度(基准:米)
    let length = |factor| (UnitCategory::Length, factor, 0.0);
    let area = |factor| (UnitCategory::Area, factor, 0.0);
    let volume = |factor| (UnitCategory::Volume, factor, 0.0);
    let mass = |factor| (UnitCategory::Mass, factor, 0.0);
    let time = |factor| (UnitCategory::Time, factor, 0.0);
    let speed = |factor| (UnitCategory::Speed, factor, 0.0);
    let data = |factor| (UnitCategory::Data, factor, 0.0);
    let energy = |factor| (UnitCategory::Energy, factor, 0.0);
    let freq = |factor| (UnitCategory::Frequency, factor, 0.0);
    Ok(match u.as_str() {
        // 长度
        "m" | "meter" | "meters" => length(1.0),
        "km" | "kilometer" | "kilometers" => length(1000.0),
        "cm" | "centimeter" => length(0.01),
        "mm" | "millimeter" => length(0.001),
        "um" | "micrometer" => length(1e-6),
        "mi" | "mile" | "miles" => length(1609.344),
        "ft" | "feet" | "foot" => length(0.3048),
        "in" | "inch" | "inches" => length(0.0254),
        "yd" | "yard" | "yards" => length(0.9144),
        "nmi" | "nautical-mile" | "nautical_mile" => length(1852.0),
        // 面积(基准:平方米)
        "m2" | "sq_m" | "sqm" => area(1.0),
        "km2" | "sq_km" => area(1e6),
        "cm2" | "sq_cm" => area(1e-4),
        "ha" | "hectare" => area(1e4),
        "acre" | "acres" => area(4046.8564224),
        "ft2" | "sq_ft" => area(0.09290304),
        "in2" | "sq_in" => area(0.00064516),
        // 体积(基准:升)
        "l" | "liter" | "liters" | "litre" => volume(1.0),
        "ml" | "milliliter" => volume(0.001),
        "m3" | "cubic_meter" => volume(1000.0),
        "cm3" | "cubic_cm" => volume(0.001),
        "gal" | "gallon" | "gallons" => volume(3.785411784),
        "qt" | "quart" => volume(0.946352946),
        "pt" | "pint" => volume(0.473176473),
        "fl_oz" | "floz" => volume(0.0295735295625),
        // 质量(基准:克)
        "g" | "gram" | "grams" => mass(1.0),
        "kg" | "kilogram" => mass(1000.0),
        "mg" | "milligram" => mass(0.001),
        "ug" | "microgram" => mass(1e-6),
        "t" | "ton" | "metric_ton" => mass(1e6),
        "lb" | "lbs" | "pound" | "pounds" => mass(453.59237),
        "oz" | "ounce" | "ounces" => mass(28.349523125),
        // 温度(基准:摄氏,非线性偏移)
        "c" | "celsius" => (UnitCategory::Temperature, 1.0, 0.0),
        "f" | "fahrenheit" => (UnitCategory::Temperature, 5.0 / 9.0, -32.0 * 5.0 / 9.0),
        "k" | "kelvin" => (UnitCategory::Temperature, 1.0, -273.15),
        // 时间(基准:秒)
        "s" | "sec" | "second" | "seconds" => time(1.0),
        "ms" | "millisecond" => time(0.001),
        "us" | "microsecond" => time(1e-6),
        "min" | "minute" | "minutes" => time(60.0),
        "h" | "hr" | "hour" | "hours" => time(3600.0),
        "d" | "day" | "days" => time(86400.0),
        // 速度(基准:m/s)
        "mps" | "m/s" => speed(1.0),
        "kmh" | "km/h" | "kph" => speed(1.0 / 3.6),
        "mph" | "mi/h" => speed(0.44704),
        "kn" | "knot" | "knots" => speed(0.514444),
        // 数据(基准:字节,二进制 1024)
        "b" | "byte" | "bytes" => data(1.0),
        "kb" | "kib" => data(1024.0),
        "mb" | "mib" => data(1024.0 * 1024.0),
        "gb" | "gib" => data(1024.0f64.powi(3)),
        "tb" | "tib" => data(1024.0f64.powi(4)),
        "bit" | "bits" => data(0.125),
        // 能量(基准:焦耳)
        "j" | "joule" | "joules" => energy(1.0),
        "kj" => energy(1000.0),
        "cal" | "calorie" => energy(4.184),
        "kcal" | "kilocalorie" => energy(4184.0),
        "wh" | "watt_hour" => energy(3600.0),
        "kwh" => energy(3_600_000.0),
        "btu" => energy(1055.05585),
        // 频率(基准:赫兹)
        "hz" | "hertz" => freq(1.0),
        "khz" => freq(1000.0),
        "mhz" => freq(1e6),
        "ghz" => freq(1e9),
        _ => return Err(ToolError::InvalidInput(format!("未知单位: {unit}"))),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    // ---- 长度 ----

    #[test]
    fn length_km_to_m() {
        assert!(approx(unit_convert(1.0, "km", "m").unwrap(), 1000.0));
    }

    #[test]
    fn length_mile_to_km() {
        assert!(approx(unit_convert(1.0, "mi", "km").unwrap(), 1.609344));
    }

    #[test]
    fn length_inch_to_cm() {
        assert!(approx(unit_convert(1.0, "in", "cm").unwrap(), 2.54));
    }

    // ---- 质量 ----

    #[test]
    fn mass_kg_to_lb() {
        assert!(approx(
            unit_convert(1.0, "kg", "lb").unwrap(),
            2.20462262185
        ));
    }

    #[test]
    fn mass_oz_to_g() {
        assert!(approx(unit_convert(1.0, "oz", "g").unwrap(), 28.349523125));
    }

    // ---- 温度(非线性)----

    #[test]
    fn temp_c_to_f() {
        assert!(approx(unit_convert(0.0, "c", "f").unwrap(), 32.0));
        assert!(approx(unit_convert(100.0, "c", "f").unwrap(), 212.0));
    }

    #[test]
    fn temp_f_to_c() {
        assert!(approx(unit_convert(32.0, "f", "c").unwrap(), 0.0));
        assert!(approx(unit_convert(212.0, "f", "c").unwrap(), 100.0));
    }

    #[test]
    fn temp_c_to_k() {
        assert!(approx(unit_convert(0.0, "c", "k").unwrap(), 273.15));
        assert!(approx(unit_convert(-273.15, "c", "k").unwrap(), 0.0));
    }

    #[test]
    fn temp_k_to_f() {
        assert!(approx(unit_convert(300.0, "k", "f").unwrap(), 80.33));
    }

    // ---- 数据 ----

    #[test]
    fn data_kb_to_mb() {
        assert!(approx(unit_convert(1024.0, "kb", "mb").unwrap(), 1.0));
    }

    #[test]
    fn data_gb_to_mb() {
        assert!(approx(unit_convert(1.0, "gb", "mb").unwrap(), 1024.0));
    }

    // ---- 时间/速度/能量/频率 ----

    #[test]
    fn time_h_to_s() {
        assert!(approx(unit_convert(1.0, "h", "s").unwrap(), 3600.0));
    }

    #[test]
    fn speed_kmh_to_mps() {
        assert!(approx(unit_convert(3.6, "kmh", "mps").unwrap(), 1.0));
    }

    #[test]
    fn energy_kwh_to_j() {
        assert!(approx(unit_convert(1.0, "kwh", "j").unwrap(), 3_600_000.0));
    }

    #[test]
    fn freq_ghz_to_mhz() {
        assert!(approx(unit_convert(1.0, "ghz", "mhz").unwrap(), 1000.0));
    }

    // ---- 错误 ----

    #[test]
    fn unknown_unit_rejected() {
        assert!(unit_convert(1.0, "foo", "bar").is_err());
    }

    #[test]
    fn cross_category_rejected() {
        assert!(unit_convert(1.0, "m", "kg").is_err(), "长度转质量应拒绝");
        assert!(unit_convert(1.0, "c", "hz").is_err(), "温度转频率应拒绝");
    }

    #[test]
    fn same_unit_identity() {
        assert!(approx(unit_convert(42.0, "m", "m").unwrap(), 42.0));
    }
}
