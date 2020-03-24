# git-profile

*git-profile* is a simple user profile manager for *git*. It lets you set-up multiple user profiles for git & switch
between them, generate remote URLs and more. If you ever have to manage multiple identities with git, *git-profile* can make
your life easier.

## Install

If you have Cargo installed, run `cargo install git-profile`. You can also grab a pre-built (Mac) binaries from the releases page.
It should be installed on your path as `git-profile`. The `git-` prefix allows it to be called like any other git command.

## Usage

### Create a profile

At a minimum you need a profile name (best to keep this quite short), your author name and author email.

`git profile new github 'Forename Surname' 'open-source@personal.com'`

Create a profile with a custom URL scheme:

`git profile new github-work 'Forename Surname' 'forename@work.com' --username CompanyName --remote 'git@github.com-work:{{username}}/{{project}}.git'`

### Switch profiles

The `use` command switches you between profiles. This sets the profile for the repository in your current working directory.

`git profile use github-work`

`git profile use open-source`

### List profiles

List all the profiles. An asterisk will appear next to the currently enabled profile.

`git profile ls`

### Use with ssh config

If your different profiles each have different ssh key-pairs set-up,
you can configure git-profile to use them via shared ssh-hosts and usernames:

`~/.ssh/config`:

```ini
# WORK GITHUB SSH CONFIG
Host work.github.com
   HostName github.com
   IdentityFile ~/.ssh/work_github_rsa
   PreferredAuthentications publickey
   IdentitiesOnly yes

# OPEN SOURCE GITHUB SSH CONFIG
Host oss.github.com
    Hostname github.com
    IdentityFile ~/.ssh/oss_github_rsa
    PreferredAuthentications publickey
    IdentitiesOnly yes
```

`~/.git_profiles`:

```toml
[gh-work]
author = 'Forename Surname (at work)'
email = 'me@work.com'
username = 'MyWorkGithubOrganization'
url = 'git@work.github.com:{{username}}/{{project}}.git'

[open-source]
author = 'Forename Surname'
email = 'my.open.source.contribs@example.com'
username = 'my-oss-github-account'
url = 'git@oss.github.com:{{username}}/{{project}}.git'
```

You can then `git clone ...` from using the correc key like ...

### Generate remote URL

git-profile can be used to generate remote URLs for your repos. This can be helpful if you have a complicated SSH
set-up that uses custom domains to use the right keys. Or just to save you having to navigate around GitHub Web
and copy and paste remote URLs.

Generate a URL for a given project name:

`git profile url project-name`

Use `-p <profile-name>` to generate using a different profile.

`git profile url -p github-work your-project`

This is particularly handy when used in a sub-shell and combined with `git-remote`:

`git remote add origin $(git profile url -p github-work my-work-project)`

### Generate author string

`git profile author` => 'Forename Surname \<your@email.address\>'

Can be used to easily fix commits when you've committed under the wrong profile:

```sh
git commit -m "Committing with the wrong user"
git profile use github-work
git commit --amend --author $(git profile author)
```

### Edit profiles

`git profile edit` opens your `.git_profiles` in `$EDITOR`. Defaults to `vim` if you don't have `$EDITOR` set.

## Status

*git-profile is in early development*. It's solves most of my major issues with using multiple identities with git, but it's by no means perfect.
If you run into a bug or have a feature request, please open an issue. It should work fine on Mac & Linux, it probably won't work as-is on Windows.

## License

MIT license, see `./LICENSE`.
