#!/usr/bin/env bash
cat << EOF
Download the appropriate file for your Operating System below, and don't forget to rename it to remove the prefix for easier use.

macOS: [\`apple-archiver\`](https://github.com/Assistant/archiver/releases/download/v${1}/apple-archiver) → \`archiver\`
Linux: [\`linux-archiver\`](https://github.com/Assistant/archiver/releases/download/v${1}/linux-archiver) → \`archiver\`
Windows: [\`windows-archiver.exe\`](https://github.com/Assistant/archiver/releases/download/v${1}/windows-archiver.exe) → \`archiver.exe\`

Check out the [\`README\`](https://github.com/Assistant/archiver#readme) for instructions on how to use this program.

EOF
cat changelogs/v${1}.md