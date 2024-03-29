pub fn get_gdb_helper() -> String {
    r#"#!/bin/sh

set -eu

OUTPUT=$(mktemp --quiet)

cat << EOF | gdb --pid="$1"
set pagination off
set variable \$env = (char**) __environ
set variable \$i=0
while (\$env[\$i] != 0)
  set variable \$pos=0
  set variable \$char=1
  while (\$char != 0)
    set variable \$char=\$env[\$i][\$pos++]
    append binary value $OUTPUT \$char
  end
  set \$i = \$i+1
end
EOF

cat "$OUTPUT"
rm "$OUTPUT"
"#
    .to_string()
}
