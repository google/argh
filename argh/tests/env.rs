use argh::FromArgs;

#[test]
fn test_env_var_option() {
    #[derive(FromArgs, PartialEq, Debug)]
    /// Reach new heights.
    struct GoUp {
        /// whether or not to jump
        #[argh(switch, short = 'j')]
        jump: bool,

        /// how high to go
        #[argh(option, env = "GO_UP_HEIGHT")]
        height: usize,

        /// optional pilot nickname
        #[argh(option, env = "GO_UP_PILOT")]
        pilot_nickname: Option<String>,
    }

    // Case 1: CLI args take precedence
    std::env::set_var("GO_UP_HEIGHT", "10");
    std::env::set_var("GO_UP_PILOT", "Maverick");
    let up = GoUp::from_args(&["cmdname"], &["--height", "5"]).expect("failed go_up");
    assert_eq!(up, GoUp { jump: false, height: 5, pilot_nickname: Some("Maverick".to_string()) });
    std::env::remove_var("GO_UP_HEIGHT");
    std::env::remove_var("GO_UP_PILOT");

    // Case 2: Env var fallback works for required
    std::env::set_var("GO_UP_HEIGHT", "20");
    let up = GoUp::from_args(&["cmdname"], &[]).expect("failed go_up env");
    assert_eq!(up, GoUp { jump: false, height: 20, pilot_nickname: None });
    std::env::remove_var("GO_UP_HEIGHT");

    // Case 3: Env var fallback works for optional
    std::env::set_var("GO_UP_HEIGHT", "30");
    std::env::set_var("GO_UP_PILOT", "Goose");
    let up = GoUp::from_args(&["cmdname"], &[]).expect("failed go_up env opt");
    assert_eq!(up, GoUp { jump: false, height: 30, pilot_nickname: Some("Goose".to_string()) });
    std::env::remove_var("GO_UP_HEIGHT");
    std::env::remove_var("GO_UP_PILOT");

    // Case 4: Missing both errors out
    let res = GoUp::from_args(&["cmdname"], &[]);
    assert!(res.is_err());
}
