# Fetch from remote origin using jj
pull:
    jj git fetch --remote origin

# Push to github
push-github:
    jj git push --tracked --remote origin

# Push to codeberg
push-codeberg:
    jj git push --tracked --remote codeberg

# Push to both github and codeberg
push: push-github push-codeberg

# Move changes to develop
move:
    jj b move --to @- develop

# Remove objects from older versions
tidy:
    cargo sweep --installed
