#[derive(serde::Serialize)]
pub struct Answer {
    pub code: u32,
    pub msg: String,
}

pub fn success() -> Answer {
    Answer {
        code: 0,
        msg: String::from("Success"),
    }
}

pub fn not_found() -> Answer {
    Answer {
        code: 1,
        msg: String::from("Error, endpoint does not exists"),
    }
}

pub fn cant_auth() -> Answer {
    Answer {
        code: 2,
        msg: String::from("Error, could not authenticate"),
    }
}

pub fn position_cant_update() -> Answer {
    Answer {
        code: 3,
        msg: String::from("Error, could not update position"),
    }
}

pub fn hash_cant_query() -> Answer {
    Answer {
        code: 4,
        msg: String::from("Error, hash not available"),
    }
}

pub fn binary_cant_create() -> Answer {
    Answer {
        code: 5,
        msg: String::from("Error, could not create binary data"),
    }
}

pub fn position_cant_query() -> Answer {
    Answer {
        code: 6,
        msg: String::from("Error, could not get position"),
    }
}

pub fn cant_register() -> Answer {
    Answer {
        code: 7,
        msg: String::from("Error, could not register"),
    }
}

pub fn cant_login() -> Answer {
    Answer {
        code: 8,
        msg: String::from("Error, could not login"),
    }
}

pub fn audiobooks_cant_query() -> Answer {
    Answer {
        code: 9,
        msg: String::from("Error, could not query audiobooks"),
    }
}
