#[macro_export]
macro_rules! register {
    ($type:path, $hook:path) => {
        $crate::inventory::submit! {
            #![crate = $crate]
            $type::new($hook, 0, false)
        }
    };

    ($type:path, $hook:path, $weight:expr) => {
        $crate::inventory::submit! {
            #![crate = $crate]
            $type::new($hook, $weight, false)
        }
    };

    ($type:path, $hook:path, $weight:expr, $default:expr) => {
        $crate::inventory::submit! {
            #![crate = $crate]
            $type::new($hook, $weight, $default)
        }
    };
}
