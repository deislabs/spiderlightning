# Contributing Guide

* [How to Help](#contributing-guide)
  * [Find an Issue](#find-an-issue)
  * [Ask for Help](#ask-for-help)
  * [Pull Request Lifecycle](#pull-request-lifecycle)
  * [Development Environment Setup](#development-environment-setup)
  * [Sign Your Commits](#sign-your-commits)
  * [Pull Request Checklist](#pull-request-checklist)

## How to Help

Welcome! We are glad that you want to contribute to our project! 💖

As you get started, you are in the best position to give us feedback on areas of our project that we need help with!

If anything doesn't make sense, or doesn't work when you run it, please open a bug report and let us know!

### Find an Issue

We have good first issues for new contributors and more involved issues suitable for seasoned contributors. [1️⃣ good first issue](https://github.com/deislabs/spiderlightning/issues?q=is%3Aopen+is%3Aissue+label%3A%221%EF%B8%8F%E2%83%A3+good+first+issue%22+first+issue%22) has extra information to help you make your first contribution. [🧩 more involved issue](https://github.com/deislabs/spiderlightning/issues?q=is%3Aopen+is%3Aissue+label%3A%22%F0%9F%A7%A9+more+involved+issue%22+) are issues suitable for someone who isn't a core maintainer and is good to move onto after your first pull request.

We will always try to have at least some issues with these labels, but, if there are not, feel free to file an issue asking if there is any place that could use some collaboration — in the future, we plan to have a Zulip/Slack/Gitter/Discord channel to make this process smoother.

Once you see an issue that you'd like to work on, please post a comment saying that you want to work on it. Something like "I want to work on this" is fine.

### Ask for Help

The best way to reach us with a question when contributing is to ask on the original github issue.

### Pull Request Lifecycle

If you are working on something that does not have an issue associated with it, your first step before making a PR should be to make an issue and explain the problem and motivation for your proposed change. Once that's done, make a PR from your feature branch (usually named `<your-username>/<feature-name>`) onto `main`. After that, someone will be assigned to review your PR.

> Note: Prior to making a PR, it's good practice to always run `cargo fmt`, and `cargo clippy`.

### Development Environment Setup

⚠️ **Explain how to set up a development environment**

### Sign Your Commits

In this repository, we require commits to be signed. If you don't know what that means, we highly recommend checking out [this](https://docs.github.com/en/authentication/managing-commit-signature-verification/signing-commits) tutorial on how to sign your own commits.

If you've already made commits and they haven't been signed, you'll have to ammend those commits with something like:
```sh
git rebase --exec 'git commit --amend --no-edit -n -S' -i <commit-hash>
```

> Note: Please, make sure that the work up until `<commit-hash>` is your work to avoid re-signing someone else's work.

#### Developer Certificate of Origin (DCO)
Licensing is important to open source projects. It provides some assurances that
the software will continue to be available based under the terms that the
author(s) desired. We require that contributors sign off on commits submitted to
our project's repositories. The [Developer Certificate of Origin
(DCO)](https://probot.github.io/apps/dco/) is a way to certify that you wrote and
have the right to contribute the code you are submitting to the project.

You sign-off by adding the following to your commit messages. Your sign-off must
match the git user and email associated with the commit.

    This is my commit message

    Signed-off-by: Your Name <your.name@example.com>

Git has a `-s` command line option to do this automatically:

    git commit -s -m 'This is my commit message'

If you forgot to do this and have not yet pushed your changes to the remote
repository, you can amend your commit with the sign-off by running 

    git commit --amend -s 
