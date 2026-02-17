# Mithrandir
This is a simple small package manager called Mithrandir.
It's nothing much, but i think it has something special. It is very customizable.

On the recipe.json you can choose the programs the package manager will execute to install certain packages. This allows the package manager to install binaries, compile from source ,install other files like .deb,.rpm etc.

But this works in theory. It may not be very practical, but it is very usefull if you want to learn Linux.
If you are a beginner don't install Mithrandir, just like Gandalf, Mithrandir is powerful but can be defeated by a Balrog on your PC.

I hope you like it.

Anonymous King.

## Package index over HTTP / GitHub
Mithrandir downloads package metadata from a web server instead of loading a local `packages.json` file.

Priority order:

1. `MTR_PACKAGES_URL` (direct URL)
2. `MTR_PACKAGES_GITHUB_REPO` (GitHub repo shortcut)
3. Fallback: `http://127.0.0.1:8080/packages.json`

### Quick local server

```bash
python3 -m http.server 8080
MTR_PACKAGES_URL="http://127.0.0.1:8080/packages.json" ./target/release/mtr node
```

### GitHub repo workflow (easy PR-based edits)

Keep `packages.json` in a GitHub repository and update it through pull requests.
Mithrandir can read it directly from `raw.githubusercontent.com`:

```bash
export MTR_PACKAGES_GITHUB_REPO="OWNER/REPO"
export MTR_PACKAGES_GITHUB_BRANCH="main"      # optional, default: main
export MTR_PACKAGES_GITHUB_PATH="packages.json"  # optional, default: packages.json
./target/release/mtr node
```

If you already have a raw URL, you can still use:

```bash
export MTR_PACKAGES_URL="https://raw.githubusercontent.com/OWNER/REPO/main/packages.json"
./target/release/mtr node
```
