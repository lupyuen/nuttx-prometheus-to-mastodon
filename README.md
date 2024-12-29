![(Experimental) Mastodon Server for Apache NuttX Continuous Integration (macOS Rancher Desktop)](https://lupyuen.github.io/images/mastodon-register7.png)

# Continuous Integration for Apache NuttX RTOS: Publish Failed Builds from Prometheus to Mastodon

Read the article...

- [__"(Experimental) Mastodon Server for Apache NuttX Continuous Integration (macOS Rancher Desktop)"__](https://lupyuen.org/articles/mastodon.html)

To Post the Failed NuttX Jobs from Prometheus to Mastodon: [run.sh](run.sh)

```bash
## Set the Access Token for Mastodon
## https://docs.joinmastodon.org/client/authorized/#token
## export MASTODON_TOKEN=...
. ../mastodon-token.sh

## Post the Failed Jobs from Prometheus to Mastodon
cargo run
```
