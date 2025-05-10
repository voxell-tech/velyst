use paste::paste;
use typst::diag::HintedString;
use typst::foundations::Symbol;
use typst::foundations::{
    Args, Array, Bytes, Content, Datetime, Dict, Duration, FromValue,
    Func, Label, Module, Scope, Smart, Str, Styles, Type, Version,
};
use typst::layout::{Abs, Angle, Em, Fr, Length, Ratio, Rel};
use typst::visualize::{Color, Gradient, Tiling};

pub trait UnitExt: Sized {
    fn length(self) -> Length;
    fn rel(self) -> Rel;

    fn smart_length(self) -> Smart<Length> {
        Smart::Custom(self.length())
    }

    fn smart_rel(self) -> Smart<Rel> {
        Smart::Custom(self.rel())
    }
}

impl UnitExt for Abs {
    fn length(self) -> Length {
        Length::from(self)
    }

    fn rel(self) -> Rel {
        Rel::from(self)
    }
}

impl UnitExt for Em {
    fn length(self) -> Length {
        Length::from(self)
    }

    fn rel(self) -> Rel {
        Rel::from(self)
    }
}

/// Implement [ScopeExt::get_value()] and [ScopeExt::get_value_unchecked()] function for given values.
macro_rules! fn_get_value {
    ($(($fn_name:ident, $get_ty:ty),)+) => {
        $(
            paste! {
                /// Clone a variable and cast it into the final value.
                ///
                /// See [ScopeExt::get_value()].
                fn $fn_name(&self, var: &str) -> Result<$get_ty, ScopeError> {
                    self.get_value(var)
                }

                /// Clone a variable and cast it into the final value _unchecked_.
                ///
                /// See [ScopeExt::get_value_unchecked()].
                fn [<$fn_name _unchecked>](&self, var: &str) -> $get_ty {
                    self.get_value_unchecked(var)
                }
            }
        )+
    };
}

pub trait ScopeExt {
    /// Clone a variable and cast it into the final value _unchecked_.
    ///
    /// # Panic
    ///
    /// - Variable does not exists.
    /// - Variable type does not match the desired value type.
    fn get_value_unchecked<T: FromValue>(&self, var: &str) -> T;

    fn get_value<T: FromValue>(
        &self,
        var: &str,
    ) -> Result<T, ScopeError>;

    fn_get_value!(
        (get_bool, bool),
        (get_int, i64),
        (get_float, f64),
        (get_len, Length),
        (get_angle, Angle),
        (get_ratio, Ratio),
        (get_relative, Rel<Length>),
        (get_fraction, Fr),
        (get_color, Color),
        (get_gradient, Gradient),
        (get_tiling, Tiling),
        (get_symbol, Symbol),
        (get_version, Version),
        (get_str, Str),
        (get_bytes, Bytes),
        (get_label, Label),
        (get_datetime, Datetime),
        (get_duration, Duration),
        (get_content, Content),
        (get_styles, Styles),
        (get_array, Array),
        (get_dict, Dict),
        (get_func, Func),
        (get_args, Args),
        (get_type, Type),
        (get_module, Module),
    );
}

impl ScopeExt for Scope {
    fn get_value_unchecked<T: FromValue>(&self, var: &str) -> T {
        self.get(var).unwrap().read().clone().cast::<T>().unwrap()
    }

    fn get_value<T: FromValue>(
        &self,
        var: &str,
    ) -> Result<T, ScopeError> {
        self.get(var).ok_or(ScopeError::VariableNotFound).and_then(
            |value| {
                value
                    .read()
                    .clone()
                    .cast::<T>()
                    .map_err(ScopeError::ValueCastFailed)
            },
        )
    }
}

#[derive(Debug)]
pub enum ScopeError {
    VariableNotFound,
    ValueCastFailed(HintedString),
}

impl std::fmt::Display for ScopeError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            ScopeError::VariableNotFound => {
                f.pad("Variable not found!")
            }
            ScopeError::ValueCastFailed(hinted_string) => write!(
                f,
                "Cast fail! {}\n{}",
                hinted_string.message(),
                hinted_string.hints().join("\n")
            ),
        }
    }
}
