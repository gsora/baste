# baste: a dumb pastebin service

Repository: https://github.com/gsora/baste

If you own a secret key, you can paste stuff on here.

If you don't you can only read unlisted pastebins.

No directory, no browsing, go snooping on pastebin.com :^)

## How-to

Look at `src/config.rs` to understand how to configure this - hint: it's all through
env variables.

Upload happens by means of multipart HTTP form, and the only file getting uploaded is
what's in `baste_file` form field.

Example:

```sh
curl https://your-paste.wow/paste -X POST -H "X-Secret-Token: {lmao get fsck'd}" -F baste_file=@obsd-sendbug
correct-horse-battery-staple

https://your-paste.wow/correct-horse-battery-staple
```
