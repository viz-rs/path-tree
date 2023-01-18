use path_tree::*;

#[test]
fn github_nodes() {
    let mut node = Node::<usize>::new(NodeKind::String(b"/".to_vec()), None);

    let mut n = node.insert_bytes(b"/");
    n = n.insert_parameter(Kind::Normal);
    n = n.insert_bytes(b"/");
    n.insert_parameter(Kind::Normal);

    node.insert_bytes(b"/api");
    node.insert_bytes(b"/about");
    node.insert_bytes(b"/login");
    node.insert_bytes(b"/signup");
    node.insert_bytes(b"/pricing");

    node.insert_bytes(b"/features");
    node.insert_bytes(b"/features/actions");
    node.insert_bytes(b"/features/packages");
    node.insert_bytes(b"/features/security");
    node.insert_bytes(b"/features/codespaces");
    node.insert_bytes(b"/features/copilot");
    node.insert_bytes(b"/features/code-review");
    node.insert_bytes(b"/features/issues");
    node.insert_bytes(b"/features/discussions");

    node.insert_bytes(b"/enterprise");
    node.insert_bytes(b"/team");
    node.insert_bytes(b"/customer-stories");
    node.insert_bytes(b"/sponsors");
    node.insert_bytes(b"/readme");
    node.insert_bytes(b"/topics");
    node.insert_bytes(b"/trending");
    node.insert_bytes(b"/collections");
    node.insert_bytes(b"/search");
    node.insert_bytes(b"/pulls");
    node.insert_bytes(b"/issues");
    node.insert_bytes(b"/marketplace");
    node.insert_bytes(b"/explore");

    node.insert_bytes(b"/sponsors/explore");
    node.insert_bytes(b"/sponsors/accounts");
    let n = node.insert_bytes(b"/sponsors/");
    n.insert_parameter(Kind::Normal);

    node.insert_bytes(b"/about/careers");
    node.insert_bytes(b"/about/press");
    node.insert_bytes(b"/about/diversity");

    node.insert_bytes(b"/settings");
    node.insert_bytes(b"/settings/admin");
    node.insert_bytes(b"/settings/appearance");
    node.insert_bytes(b"/settings/accessibility");
    node.insert_bytes(b"/settings/notifications");

    node.insert_bytes(b"/settings/billing");
    node.insert_bytes(b"/settings/billing/plans");
    node.insert_bytes(b"/settings/security");
    node.insert_bytes(b"/settings/keys");
    node.insert_bytes(b"/settings/organizations");

    node.insert_bytes(b"/settings/blocked_users");
    node.insert_bytes(b"/settings/interaction_limits");
    node.insert_bytes(b"/settings/code_review_limits");

    node.insert_bytes(b"/settings/repositories");
    node.insert_bytes(b"/settings/codespaces");
    node.insert_bytes(b"/settings/deleted_packages");
    node.insert_bytes(b"/settings/copilot");
    node.insert_bytes(b"/settings/pages");
    node.insert_bytes(b"/settings/replies");

    node.insert_bytes(b"/settings/security_analysis");

    node.insert_bytes(b"/settings/installations");
    node.insert_bytes(b"/settings/reminders");

    node.insert_bytes(b"/settings/security-log");
    node.insert_bytes(b"/settings/sponsors-log");

    node.insert_bytes(b"/settings/apps");
    node.insert_bytes(b"/settings/developers");
    node.insert_bytes(b"/settings/tokens");

    node.insert_bytes(b"/404");
    node.insert_bytes(b"/500");
    node.insert_bytes(b"/503");

    assert_eq!(
        format!("{node:?}"),
        r#"
/
├── 404
├── 50
│   ├── 0
│   └── 3
├── a
│   ├── bout
│   │   └── /
│   │       ├── careers
│   │       ├── diversity
│   │       └── press
│   └── pi
├── c
│   ├── ollections
│   └── ustomer-stories
├── e
│   ├── nterprise
│   └── xplore
├── features
│   └── /
│       ├── actions
│       ├── co
│       │   ├── de
│       │   │   ├── -review
│       │   │   └── spaces
│       │   └── pilot
│       ├── discussions
│       ├── issues
│       ├── packages
│       └── security
├── issues
├── login
├── marketplace
├── p
│   ├── ricing
│   └── ulls
├── readme
├── s
│   ├── e
│   │   ├── arch
│   │   └── ttings
│   │       └── /
│   │           ├── a
│   │           │   ├── ccessibility
│   │           │   ├── dmin
│   │           │   └── pp
│   │           │       ├── earance
│   │           │       └── s
│   │           ├── b
│   │           │   ├── illing
│   │           │   │   └── /plans
│   │           │   └── locked_users
│   │           ├── co
│   │           │   ├── de
│   │           │   │   ├── _review_limits
│   │           │   │   └── spaces
│   │           │   └── pilot
│   │           ├── de
│   │           │   ├── leted_packages
│   │           │   └── velopers
│   │           ├── in
│   │           │   ├── stallations
│   │           │   └── teraction_limits
│   │           ├── keys
│   │           ├── notifications
│   │           ├── organizations
│   │           ├── pages
│   │           ├── re
│   │           │   ├── minders
│   │           │   └── p
│   │           │       ├── lies
│   │           │       └── ositories
│   │           ├── s
│   │           │   ├── ecurity
│   │           │   │   ├── -log
│   │           │   │   └── _analysis
│   │           │   └── ponsors-log
│   │           └── tokens
│   ├── ignup
│   └── ponsors
│       └── /
│           ├── accounts
│           ├── explore
│           └── :
├── t
│   ├── eam
│   ├── opics
│   └── rending
└── :
    └── /
        └── :
"#
    );
}
