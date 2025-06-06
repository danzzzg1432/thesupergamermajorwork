pub mod level;
pub mod room;

use starlark::syntax::{Dialect, DialectTypes};

pub const DIALECT: Dialect = Dialect {
    enable_load: false,
    enable_keyword_only_arguments: true,
    enable_positional_only_arguments: true,
    enable_types: DialectTypes::Enable,
    enable_top_level_stmt: true,
    enable_f_strings: true,
    ..Dialect::Standard
};
