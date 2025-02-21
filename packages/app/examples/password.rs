use dirtybase_contract::auth::AuthUser;

fn main() {
    let mut auth_user = AuthUser::default();
    let token = auth_user.generate_token();
    println!("token: {}", &token);
    println!("is token valid: {}", auth_user.validate_token(&token));
    auth_user.rotate_salt();
    println!("is token valid 2: {}", auth_user.validate_token(&token));
}
