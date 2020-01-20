const AIR_DENSITY_SEA_LEVEL: f32 = 1.225;
const AIR_GAS: f32 = 287.1;
const ABSOLUTE_NULL_TEMPERATURE: f32 = -273.15;

pub const fn calculate_eas_from_ias(speed: f32, scale: f32) -> f32 {
    speed * scale
}

pub const fn calculate_tas_from_eas(speed_equivalent: f32, air_pressure: f32, air_temperature: f32) -> f32 {
    speed_equivalent * (AIR_DENSITY_SEA_LEVEL / air_density(air_pressure, air_temperature))

}

pub const fn air_density(static_pressure: f32, temperature: f32) -> f32 {
    static_pressure / AIR_GAS * (temperature - ABSOLUTE_NULL_TEMPERATURE)
}
