#!/usr/bin/env sh

# lol cringe whatever

set -e

name=ghcr.io/necauqua/changelogs:v1

docker build -t $name .

# digest only appears after push, oh well
docker push $name

digest=$(docker inspect $name --format '{{index .RepoDigests 0}}')
sed -ri "s|docker://.*|docker://$digest|g" */action.yml

git add */action.yml
git commit # open the editor for commit msg or to cancel

# now reset the v1 branch to main
git checkout v1
git reset --hard main
git checkout main

# and push too, meh
git push github main v1
