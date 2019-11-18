git-profile
==========

*git-profile* is a simple user profile manager for *git*. It lets you easily set-up user profiles for git & switch
between them, generate remote URLs and more. *git-profile* can help if you:

  * Do open-source work & 9-to-5 work from the same machine
  * Freelancer & have lots of different organisations to work with
  * Use multiple git hosts at once
  * Have an SSH setup that needs you to use strange URLs to push with the right keys (`github.com-{username}` is a
    common pattern seen on StackOverflow etc.)



## Usage

*git-profile* should be installed as a binary named `git-profile`. The `git-` suffix means we can use it like any other
git command.

### Create a Profile
At a minimum you need a profile name (best to keep this quite short), your author name and author email.
`git profile add github 'Forename Surname' 'open-source@personal.com'`

Create a profile with a custom URL scheme:
`git profile add github-work 'Forname Surname' 'forename@work.com' --username CompanyName --remote 'git@github.com-work:{{username}}/{{project}}.git'`

### Switch profiles
The `use` command lets you trivially switch between profiles:
`git profile use github-work`
`git profile use freelance-company-39`

This sets the local `git config user.name` and `git config user.email` to the author name & email in that profile.

### List all your profiles
List all the profiles. An asterisk will appear next to the current enabled profile.
`git profile ls`

### Generate a URL
git-profile can be used to generate remote URLs for your repos. This can be helpful if you have a complicated SSH
set-up that uses custom domains to use the right keys. Or just to save you having to navigate around GitHub Web
and copy and paste remote URLs.

Generate a URL for a given project name:
`git profile url project-name`

Use `-p <profile-name>` to generate using a different profile.
`git profile url -p github-work your-project`

This is particularly handy when used in a sub-shell and combined with `git-remote`. 
`git remote add origin $(git profile url -p github-work my-work-project)`

### Generate author string
`git profile author` => 'Forname Surname <your@email.address>

### Edit profiles in an editor
`git profile edit` opens your `.git_profiles` in `$EDITOR`

Can be used to easily fix commits when you've committed under the wrong profile:
```
git commit -m "Committing with the wrong user"
git profile use github-work
git commit --amend --author $(git profile author)
```

## Status

*git-profile is in early development*. Use early stage software with caution.


## License

MIT license, see `./LICENSE`.
