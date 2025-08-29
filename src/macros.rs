#[macro_export]
macro_rules! or_continue {
    ($e:expr) => {
        match $e {
            Some(x) => x,
            None => continue
        }
    };
}

#[macro_export]
macro_rules! continue_unless {
    ($e:expr) => {
        if !$e {
            continue;
        }
    };
}

#[macro_export]
macro_rules! return_unless {
    ($e:expr, $r:expr) => {
        if !$e {
            return $r;
        }
    };
}

#[macro_export]
macro_rules! or_return {
    ($e:expr, $r:expr) => {
        match $e {
            Some(x) => x,
            None => return $r
        }
    }
}

#[macro_export]
macro_rules! print_iter {
    ($i:expr) => {{
        let mut write_buf = String::new();

        ::core::iter::Iterator::for_each($i, |s| write_buf.push_str(&format!("{}, ", s)));

        write_buf.truncate(write_buf.len().saturating_sub(2));

        println!("[{}]", write_buf);
    }};
    ($i:ident) => {$i = {
        let mut write_buf = String::new();

        ::core::iter::Iterator::for_each($i, |s| write_buf.push_str(&format!("{}, ", s)));

        write_buf.truncate(write_buf.len().saturating_sub(2));

        println!("[{}]", write_buf);

        $i
    }}
}

#[macro_export]
macro_rules! is_kind_of {
    ($e:expr, $p:pat) => {
        match $e {
            $p => true,
            _ => false
        }
    }
}

#[macro_export]
macro_rules! stat {
    () => {
        println!("[{}:{}:{}]", file!(), line!(), column!());
    };
    ($msg:literal) => {
        println!("[{}:{}:{}]: {}", file!(), line!(), column!(), $msg);
    }
}

#[macro_export]
macro_rules! int_to_bool {
    ($e:expr) => {
        match $e {
            0 => Some(false),
            1 => Some(true),
            _ => None
        }
    }
}