use typst::{
    foundations::{
        Args, Array, Bytes, Content, Datetime, Dict, Duration, FromValue, Func, Label, Module,
        Plugin, Scope, Smart, Str, Styles, Type, Version,
    },
    layout::{Abs, Angle, Em, Fr, Length, Ratio, Rel},
    symbols::Symbol,
    visualize::{Color, Gradient, Pattern},
};

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

macro_rules! fn_get_unchecked {
    ($fn_name:ident, $get_ty:ty) => {
        /// Clone a variable and cast it into the final value _unchecked_.
        ///
        /// See [ScopeExt::get_unchecked()].
        fn $fn_name(&self, var: &str) -> $get_ty {
            self.get_unchecked(var)
        }
    };
}

pub trait ScopeExt {
    /// Clone a variable and cast it into the final value _unchecked_.
    ///
    /// # Panic
    ///
    /// - Variable does not exists.
    /// - Variable type does not match the desired value type.
    fn get_unchecked<T: FromValue>(&self, var: &str) -> T;

    fn_get_unchecked!(get_unchecked_bool, bool);
    fn_get_unchecked!(get_unchecked_int, i64);
    fn_get_unchecked!(get_unchecked_float, f64);
    fn_get_unchecked!(get_unchecked_len, Length);
    fn_get_unchecked!(get_unchecked_angle, Angle);
    fn_get_unchecked!(get_unchecked_ratio, Ratio);
    fn_get_unchecked!(get_unchecked_relative, Rel<Length>);
    fn_get_unchecked!(get_unchecked_fraction, Fr);
    fn_get_unchecked!(get_unchecked_color, Color);
    fn_get_unchecked!(get_unchecked_gradient, Gradient);
    fn_get_unchecked!(get_unchecked_pattern, Pattern);
    fn_get_unchecked!(get_unchecked_symbol, Symbol);
    fn_get_unchecked!(get_unchecked_version, Version);
    fn_get_unchecked!(get_unchecked_str, Str);
    fn_get_unchecked!(get_unchecked_bytes, Bytes);
    fn_get_unchecked!(get_unchecked_label, Label);
    fn_get_unchecked!(get_unchecked_datetime, Datetime);
    fn_get_unchecked!(get_unchecked_duration, Duration);
    fn_get_unchecked!(get_unchecked_content, Content);
    fn_get_unchecked!(get_unchecked_styles, Styles);
    fn_get_unchecked!(get_unchecked_array, Array);
    fn_get_unchecked!(get_unchecked_dict, Dict);
    fn_get_unchecked!(get_unchecked_func, Func);
    fn_get_unchecked!(get_unchecked_args, Args);
    fn_get_unchecked!(get_unchecked_type, Type);
    fn_get_unchecked!(get_unchecked_module, Module);
    fn_get_unchecked!(get_unchecked_plugin, Plugin);
}

impl ScopeExt for Scope {
    fn get_unchecked<T: FromValue>(&self, var: &str) -> T {
        self.get(var).cloned().unwrap().cast::<T>().unwrap()
    }
}
