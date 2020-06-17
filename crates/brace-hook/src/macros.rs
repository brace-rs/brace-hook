#[macro_export]
macro_rules! register {
    ($type:path, $hook:path) => {
        $crate::inventory::submit! {
            #![crate = $crate]
            $type::new($hook, 0)
        }
    };

    ($type:path, $hook:path, $weight:expr) => {
        $crate::inventory::submit! {
            #![crate = $crate]
            $type::new($hook, $weight)
        }
    };
}
