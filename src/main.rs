use keybase_protocol::protocol::api::MsgNotification;
use notify_rust::{Hint, Notification};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    /// Your username (to filter messages you send yourself).
    ///
    /// If not set, will attempt to use `keybase status` to find the username.
    #[structopt(long)]
    me: Option<String>,

    /// Use a specific notification icon
    #[structopt(short, long)]
    icon: Option<String>,
}

fn main() {
    let opt = Opt::from_args();

    let me = if let Some(ref me) = opt.me {
        Some(me.to_owned())
    } else {
        match std::process::Command::new("keybase")
            .arg("status")
            .arg("--json")
            .output()
        {
            Ok(out) => {
                if !out.status.success() {
                    eprintln!(
                        "failed to use `keybase status` to determine username: {}",
                        String::from_utf8_lossy(&out.stderr[..])
                    );
                    None
                } else {
                    let v: serde_json::Value = serde_json::from_slice(&out.stdout[..]).unwrap();
                    v.get("Username")
                        .and_then(|v| v.as_str())
                        .map(str::to_owned)
                }
            }
            Err(e) => {
                eprintln!(
                    "failed to use `keybase status` to determine username: {}",
                    e
                );
                None
            }
        }
    };

    let stdin = std::io::stdin();
    let api = stdin.lock();
    let api = serde_json::Deserializer::from_reader(api).into_iter::<MsgNotification>();
    for r in api {
        match r {
            Ok(msg) if msg.msg.is_some() => {
                let msg = msg.msg.unwrap();
                let channel = msg.channel.unwrap();
                let sender = msg.sender.unwrap();
                let content = msg.content.unwrap();
                if false && !msg.unread.unwrap_or(false) {
                    // already seen -- don't spam the user
                    continue;
                }

                if let Some(ref me) = me {
                    if sender.username.as_ref().unwrap() == me {
                        continue;
                    }
                }

                let mut n = Notification::new();
                n.appname("keybase")
                    .summary(channel.name.as_ref().unwrap())
                    .body(&format!(
                        "{}: {}",
                        sender.username.unwrap(),
                        content.text.unwrap().body.unwrap()
                    ))
                    .icon(opt.icon.as_ref().map(|s| &**s).unwrap_or("im-message-new"))
                    .hint(Hint::Category("im.received".to_owned()))
                    .hint(Hint::DesktopEntry("keybase".to_owned()))
                    .show()
                    .unwrap();
            }
            Ok(msg) => {
                eprintln!("notification w/o message: {:?}", msg);
            }
            Err(e) => {
                eprintln!("{:?}", e);
            }
        }
    }
}
