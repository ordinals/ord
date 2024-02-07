deploy-ordinals domain="ordinals.inscribingatlantis.com" remote="InscribingAtlantis/ord" branch="master" chain="main":
  ssh root@{{domain}} 'mkdir -p deploy \
    && apt-get update --yes \
    && apt-get upgrade --yes \
    && apt-get install --yes git rsync'
  rsync -avz deploy/checkout root@{{domain}}:deploy/checkout
  ssh root@{{domain}} 'cd deploy && ./checkout {{branch}} {{remote}} {{chain}} {{domain}}'

deploy-preview domain="preview.inscribingatlantis.com" remote="InscribingAtlantis/ord" branch="master" chain="regtest":
  ssh root@{{domain}} 'mkdir -p deploy \
    && apt-get update --yes \
    && apt-get upgrade --yes \
    && apt-get install --yes git rsync'
  rsync -avz deploy/checkout root@{{domain}}:deploy/checkout
  ssh root@{{domain}} 'cd deploy && ./checkout {{branch}} {{remote}} {{chain}} {{domain}}'
