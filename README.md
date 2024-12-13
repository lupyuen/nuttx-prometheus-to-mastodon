# Continuous Integration for Apache NuttX RTOS: Publish Failed Builds from Prometheus to Mastodon

See [run.sh](run.sh)

```bash
## Set the Access Token for Mastodon
## https://docs.joinmastodon.org/client/authorized/#token
## export MASTODON_TOKEN=...
. ../mastodon-token.sh

## Post the Failed Jobs from Prometheus to Mastodon
cargo run
```
