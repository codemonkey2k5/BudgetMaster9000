# Security Policy

## What Budget Master 9000 protects

Budget Master 9000 is a **local desktop app**. Your data stays on your machine in a SQLite database.

Optional **App Lock** uses Argon2id to hash a passphrase and gates access to the UI/API. This is intended to stop **casual snooping** on a shared PC.

## What it does not protect against

- Malware running in your user session
- Someone with full access to an unlocked Windows account
- Weak passphrases (e.g. `123456`)
- Forgetting your passphrase — **there is no cloud recovery**

## Recommendations

1. Use a strong App Lock passphrase if others use your PC
2. Enable BitLocker (or full-disk encryption) on laptops
3. Export backups and store them securely
4. For thumb-drive portable mode, treat the drive like cash

## Reporting issues

Please open a private security advisory or GitHub issue describing the problem without publishing exploit details that put users at immediate risk.
