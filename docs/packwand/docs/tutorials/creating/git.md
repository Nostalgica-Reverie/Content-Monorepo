# Using packwand with Git

On Windows, line ending conversion causes the hashes to change when files are uploaded to Git, so you'll get invalid hash errors when trying to install the pack. You'll want to add a `.gitattributes` file to disable line ending conversion. See the [example pack](https://github.com/packwiz/packwiz-example-pack) for example `.gitignore` and `.gitattributes` files!

If you have existing files committed to Git, you'll need to run `git add --renormalize .` to reset line ending conversion after adding `.gitattributes`.

You'll also want a [`.packwizignore`](/reference/pack-format/packwizignore) file so that Git metadata isn't included in the pack index — though packwand ignores `.git/**`, `.gitignore`, and `.gitattributes` by default.
