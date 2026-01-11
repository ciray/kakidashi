#!/bin/bash -eu

echo "Initializing git submodules..."

git submodule init
git config -f .git/modules/aozorabunko/config core.sparseCheckout true

cat > .git/modules/aozorabunko/info/sparse-checkout << 'EOF'
/index_pages/person_all.html
/index_pages/person[0-9]*.html
/cards/*/card*.html
/cards/*/files/*.zip
EOF

git submodule update --force --checkout aozorabunko

echo "Initialized git submodules."
