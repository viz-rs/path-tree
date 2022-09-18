use path_tree::*;
// use rand::seq::SliceRandom;
//
// fn shuffle<'a>(routes: &'a [(&str, usize)]) -> PathTree<'a, usize> {
//     let mut routes = routes.to_vec();
//     let mut tree = PathTree::new("/");
//
//     routes.shuffle(&mut rand::thread_rng());
//
//     for (path, value) in routes {
//         tree.insert(path, value);
//     }
//
//     tree
// }

#[test]
fn basic() {
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/", 0);
    tree.insert("/users", 1);
    tree.insert("/users/:id", 2);
    tree.insert("/users/:id/:org", 3);
    tree.insert("/users/:userId/repos", 4);
    tree.insert("/users/:userId/repos/:id", 5);
    tree.insert("/users/:userId/repos/:id/:any*", 6);
    tree.insert(r"/\\::username", 7);
    tree.insert("/*", 8);
    tree.insert("/about", 9);
    tree.insert("/about/", 10);
    tree.insert("/about/us", 11);
    tree.insert("/users/repos/*", 12);
    tree.insert("/:action", 13);
    tree.insert("", 14);

    assert_eq!(
        format!("{:?}", &tree.node),
        r#"
/ •0
├── \:
│   └── : •7
├── about •9
│   └── / •10
│       └── us •11
├── users •1
│   └── /
│       ├── repos/
│       │   └── ** •12
│       └── : •2
│           └── /
│               ├── repos •4
│               │   └── /
│               │       └── : •5
│               │           └── /
│               │               └── ** •6
│               └── : •3
├── : •13
└── ** •8
"#
    );

    assert_eq!(
        tree.find("/"),
        Some((&0, &vec![Piece::String(b"/")], vec![]))
    );
    assert_eq!(
        tree.find("/users"),
        Some((&1, &vec![Piece::String(b"/users")], vec![]))
    );
    assert_eq!(
        tree.find("/users/foo"),
        Some((
            &2,
            &vec![
                Piece::String(b"/users/"),
                Piece::Parameter(Position::Named("id"), Kind::Normal)
            ],
            vec!["foo"]
        ))
    );
    assert_eq!(
        tree.find("/users/foo/bar"),
        Some((
            &3,
            &vec![
                Piece::String(b"/users/"),
                Piece::Parameter(Position::Named("id"), Kind::Normal),
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("org"), Kind::Normal),
            ],
            vec!["foo", "bar"]
        ))
    );
    assert_eq!(
        tree.find("/users/foo/repos"),
        Some((
            &4,
            &vec![
                Piece::String(b"/users/"),
                Piece::Parameter(Position::Named("userId"), Kind::Normal),
                Piece::String(b"/repos"),
            ],
            vec!["foo"]
        ))
    );
    assert_eq!(
        tree.find("/users/foo/repos/bar"),
        Some((
            &5,
            &vec![
                Piece::String(b"/users/"),
                Piece::Parameter(Position::Named("userId"), Kind::Normal),
                Piece::String(b"/repos/"),
                Piece::Parameter(Position::Named("id"), Kind::Normal),
            ],
            vec!["foo", "bar"]
        ))
    );
    assert_eq!(
        tree.find("/users/foo/repos/bar/"),
        Some((
            &6,
            &vec![
                Piece::String(b"/users/"),
                Piece::Parameter(Position::Named("userId"), Kind::Normal),
                Piece::String(b"/repos/"),
                Piece::Parameter(Position::Named("id"), Kind::Normal),
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("any"), Kind::ZeroOrMoreSegment),
            ],
            vec!["foo", "bar", ""]
        ))
    );
    assert_eq!(
        tree.find("/users/foo/repos/bar/baz"),
        Some((
            &6,
            &vec![
                Piece::String(b"/users/"),
                Piece::Parameter(Position::Named("userId"), Kind::Normal),
                Piece::String(b"/repos/"),
                Piece::Parameter(Position::Named("id"), Kind::Normal),
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("any"), Kind::ZeroOrMoreSegment),
            ],
            vec!["foo", "bar", "baz"]
        ))
    );
    assert_eq!(
        tree.find("/:foo"),
        Some((
            &7,
            &vec![
                Piece::String(b"/"),
                Piece::String(b":"),
                Piece::Parameter(Position::Named("username"), Kind::Normal),
            ],
            vec!["foo"]
        ))
    );
    assert_eq!(
        tree.find("/foo/bar/baz/404"),
        Some((
            &8,
            &vec![
                Piece::String(b"/"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMoreSegment),
            ],
            vec!["foo/bar/baz/404"]
        ))
    );
    assert_eq!(
        tree.find("/about"),
        Some((&9, &vec![Piece::String(b"/about")], vec![]))
    );
    assert_eq!(
        tree.find("/about/"),
        Some((&10, &vec![Piece::String(b"/about/")], vec![]))
    );
    assert_eq!(
        tree.find("/about/us"),
        Some((&11, &vec![Piece::String(b"/about/us")], vec![]))
    );
    assert_eq!(
        tree.find("/users/repos/foo"),
        Some((
            &12,
            &vec![
                Piece::String(b"/users/repos/"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMoreSegment),
            ],
            vec!["foo"]
        ))
    );
    assert_eq!(
        tree.find("/users/repos/foo/bar"),
        Some((
            &12,
            &vec![
                Piece::String(b"/users/repos/"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMoreSegment),
            ],
            vec!["foo/bar"]
        ))
    );
    assert_eq!(
        tree.find("/-foo"),
        Some((
            &13,
            &vec![
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("action"), Kind::Normal),
            ],
            vec!["-foo"]
        ))
    );
}

#[test]
fn print_github_tree() {
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/", 0);
    tree.insert("/api", 1);
    tree.insert("/about", 2);
    tree.insert("/login", 3);
    tree.insert("/signup", 4);
    tree.insert("/pricing", 5);

    tree.insert("/features", 6);
    tree.insert("/features/actions", 600);
    tree.insert("/features/packages", 601);
    tree.insert("/features/security", 602);
    tree.insert("/features/codespaces", 603);
    tree.insert("/features/copilot", 604);
    tree.insert("/features/code-review", 605);
    tree.insert("/features/issues", 606);
    tree.insert("/features/discussions", 607);

    tree.insert("/enterprise", 7);
    tree.insert("/team", 8);
    tree.insert("/customer-stories", 9);
    tree.insert("/sponsors", 10);
    tree.insert("/readme", 11);
    tree.insert("/topics", 12);
    tree.insert("/trending", 13);
    tree.insert("/collections", 14);
    tree.insert("/search", 15);
    tree.insert("/pulls", 16);
    tree.insert("/issues", 17);
    tree.insert("/marketplace", 18);
    tree.insert("/explore", 19);

    tree.insert("/sponsors/explore", 100);
    tree.insert("/sponsors/accounts", 101);
    tree.insert("/sponsors/:repo", 102);
    tree.insert("/sponsors/:repo/:user?", 103);
    tree.insert("/sponsors/:repo/+", 104);
    tree.insert("/sponsors/:repo/:user", 105);
    tree.insert("/sponsors/:repo/issues/*", 106);
    tree.insert("/sponsors/:repo/+/:file", 107);
    tree.insert("/sponsors/:repo/+/:filename.:ext", 108);

    tree.insert("/about/careers", 200);
    tree.insert("/about/press", 201);
    tree.insert("/about/diversity", 202);

    tree.insert("/settings", 20);
    tree.insert("/settings/admin", 2000);
    tree.insert("/settings/appearance", 2001);
    tree.insert("/settings/accessibility", 2002);
    tree.insert("/settings/notifications", 2003);

    tree.insert("/settings/billing", 2004);
    tree.insert("/settings/billing/plans", 2005);
    tree.insert("/settings/security", 2006);
    tree.insert("/settings/keys", 2007);
    tree.insert("/settings/organizations", 2008);

    tree.insert("/settings/blocked_users", 2009);
    tree.insert("/settings/interaction_limits", 2010);
    tree.insert("/settings/code_review_limits", 2011);

    tree.insert("/settings/repositories", 2012);
    tree.insert("/settings/codespaces", 2013);
    tree.insert("/settings/deleted_packages", 2014);
    tree.insert("/settings/copilot", 2015);
    tree.insert("/settings/pages", 2016);
    tree.insert("/settings/replies", 2017);

    tree.insert("/settings/security_analysis", 2018);

    tree.insert("/settings/installations", 2019);
    tree.insert("/settings/reminders", 2020);

    tree.insert("/settings/security-log", 2021);
    tree.insert("/settings/sponsors-log", 2022);

    tree.insert("/settings/apps", 2023);
    tree.insert("/settings/developers", 2024);
    tree.insert("/settings/tokens", 2025);

    tree.insert("/404", 21);
    tree.insert("/500", 22);
    tree.insert("/503", 23);

    tree.insert("/:org", 24);
    tree.insert("/:org/:repo", 2400);
    tree.insert("/:org/:repo/issues", 2410);
    tree.insert("/:org/:repo/issues/:id", 2411);
    tree.insert("/:org/:repo/issues/new", 2412);
    tree.insert("/:org/:repo/pulls", 2420);
    tree.insert("/:org/:repo/pull/:id", 2421);
    tree.insert("/:org/:repo/compare", 2422);
    tree.insert("/:org/:repo/discussions", 2430);
    tree.insert("/:org/:repo/discussions/:id", 2431);
    tree.insert("/:org/:repo/actions", 2440);
    tree.insert("/:org/:repo/actions/workflows/:id", 2441);
    tree.insert("/:org/:repo/actions/runs/:id", 2442);
    tree.insert("/:org/:repo/wiki", 2450);
    tree.insert("/:org/:repo/wiki/:id", 2451);
    tree.insert("/:org/:repo/security", 2460);
    tree.insert("/:org/:repo/security/policy", 2461);
    tree.insert("/:org/:repo/security/advisories", 2462);
    tree.insert("/:org/:repo/pulse", 2470);
    tree.insert("/:org/:repo/graphs/contributors", 2480);
    tree.insert("/:org/:repo/graphs/commit-activity", 2481);
    tree.insert("/:org/:repo/graphs/code-frequency", 2482);
    tree.insert("/:org/:repo/community", 2490);
    tree.insert("/:org/:repo/network", 2491);
    tree.insert("/:org/:repo/network/dependencies", 2492);
    tree.insert("/:org/:repo/network/dependents", 2493);
    tree.insert("/:org/:repo/network/members", 2494);
    tree.insert("/:org/:repo/stargazers", 2495);
    tree.insert("/:org/:repo/stargazers/yoou_know", 2496);
    tree.insert("/:org/:repo/watchers", 2497);
    tree.insert("/:org/:repo/releases", 2498);
    tree.insert("/:org/:repo/releases/tag/:id", 2499);
    tree.insert("/:org/:repo/tags", 2500);
    tree.insert("/:org/:repo/tags/:id", 2501);
    tree.insert("/:org/:repo/tree/:id", 2502);
    tree.insert("/:org/:repo/commit/:id", 2503);

    tree.insert("/new", 2504);
    tree.insert("/new/import", 2505);
    tree.insert("/organizations/new", 2506);
    tree.insert("/organizations/plan", 2507);

    tree.insert("/:org/:repo/*", 3000);
    tree.insert("/:org/:repo/releases/*", 3001);
    tree.insert("/:org/:repo/releases/download/:tag/:filename.:ext", 3002);

    assert_eq!(
        format!("{:?}", dbg!(&tree.node)),
        r#"
/ •0
├── 404 •67
├── 50
│   ├── 0 •68
│   └── 3 •69
├── a
│   ├── bout •2
│   │   └── /
│   │       ├── careers •37
│   │       ├── diversity •39
│   │       └── press •38
│   └── pi •1
├── c
│   ├── ollections •22
│   └── ustomer-stories •17
├── e
│   ├── nterprise •15
│   └── xplore •27
├── features •6
│   └── /
│       ├── actions •7
│       ├── co
│       │   ├── de
│       │   │   ├── -review •12
│       │   │   └── spaces •10
│       │   └── pilot •11
│       ├── discussions •14
│       ├── issues •13
│       ├── packages •8
│       └── security •9
├── issues •25
├── login •3
├── marketplace •26
├── new •106
│   └── /import •107
├── organizations/
│   ├── new •108
│   └── plan •109
├── p
│   ├── ricing •5
│   └── ulls •24
├── readme •19
├── s
│   ├── e
│   │   ├── arch •23
│   │   └── ttings •40
│   │       └── /
│   │           ├── a
│   │           │   ├── ccessibility •43
│   │           │   ├── dmin •41
│   │           │   └── pp
│   │           │       ├── earance •42
│   │           │       └── s •64
│   │           ├── b
│   │           │   ├── illing •45
│   │           │   │   └── /plans •46
│   │           │   └── locked_users •50
│   │           ├── co
│   │           │   ├── de
│   │           │   │   ├── _review_limits •52
│   │           │   │   └── spaces •54
│   │           │   └── pilot •56
│   │           ├── de
│   │           │   ├── leted_packages •55
│   │           │   └── velopers •65
│   │           ├── in
│   │           │   ├── stallations •60
│   │           │   └── teraction_limits •51
│   │           ├── keys •48
│   │           ├── notifications •44
│   │           ├── organizations •49
│   │           ├── pages •57
│   │           ├── re
│   │           │   ├── minders •61
│   │           │   └── p
│   │           │       ├── lies •58
│   │           │       └── ositories •53
│   │           ├── s
│   │           │   ├── ecurity •47
│   │           │   │   ├── -log •62
│   │           │   │   └── _analysis •59
│   │           │   └── ponsors-log •63
│   │           └── tokens •66
│   ├── ignup •4
│   └── ponsors •18
│       └── /
│           ├── accounts •29
│           ├── explore •28
│           └── : •30
│               └── /
│                   ├── issues/
│                   │   └── ** •34
│                   ├── : •33
│                   ├── ?? •31
│                   └── + •32
│                       └── /
│                           └── : •35
│                               └── .
│                                   └── : •36
├── t
│   ├── eam •16
│   ├── opics •20
│   └── rending •21
└── : •70
    └── /
        └── : •71
            └── /
                ├── actions •80
                │   └── /
                │       ├── runs/
                │       │   └── : •82
                │       └── workflows/
                │           └── : •81
                ├── com
                │   ├── m
                │   │   ├── it/
                │   │   │   └── : •105
                │   │   └── unity •92
                │   └── pare •77
                ├── discussions •78
                │   └── /
                │       └── : •79
                ├── graphs/co
                │   ├── de-frequency •91
                │   ├── mmit-activity •90
                │   └── ntributors •89
                ├── issues •72
                │   └── /
                │       ├── new •74
                │       └── : •73
                ├── network •93
                │   └── /
                │       ├── dependen
                │       │   ├── cies •94
                │       │   └── ts •95
                │       └── members •96
                ├── pul
                │   ├── l
                │   │   ├── /
                │   │   │   └── : •76
                │   │   └── s •75
                │   └── se •88
                ├── releases •100
                │   └── /
                │       ├── download/
                │       │   └── :
                │       │       └── /
                │       │           └── :
                │       │               └── .
                │       │                   └── : •112
                │       ├── tag/
                │       │   └── : •101
                │       └── ** •111
                ├── s
                │   ├── ecurity •85
                │   │   └── /
                │   │       ├── advisories •87
                │   │       └── policy •86
                │   └── targazers •97
                │       └── /yoou_know •98
                ├── t
                │   ├── ags •102
                │   │   └── /
                │   │       └── : •103
                │   └── ree/
                │       └── : •104
                ├── w
                │   ├── atchers •99
                │   └── iki •83
                │       └── /
                │           └── : •84
                └── ** •110
"#
    );

    dbg!(tree.find("/rust-lang/rust"));
    dbg!(tree.find("/settings"));
    dbg!(tree.find("/rust-lang/rust/actions/runs/1"));
    dbg!(tree.find("/rust-lang/rust/"));
    dbg!(tree.find("/rust-lang/rust/any"));
    dbg!(tree.find("/rust-lang/rust/releases/"));
    dbg!(tree.find("/rust-lang/rust-analyzer/releases/download/2022-09-12/rust-analyzer-aarch64-apple-darwin.gz"));
}
