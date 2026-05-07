# Auth Credential Constraints

## Goal

Add explicit username and password constraints, apply identical rules on backend and frontend, and trim credential inputs before validation and use.

## Selected Rules

- Username is trimmed before validation and persistence.
- Username length: 3-30 characters.
- Username characters: ASCII letters, numbers, `_`, `-`.
- Username must start and end with a letter or number.
- Password is trimmed before validation, hashing, and login verification.
- Password length: 8-128 characters.
- Password does not require composition rules such as uppercase, digit, or symbol.

## Research Notes

- NIST SP 800-63B and OWASP Authentication Cheat Sheet emphasize minimum length, allowing long passwords/passphrases, and avoiding outdated composition rules.
- Common username rules across large sites usually keep usernames short and ASCII-safe, with letters, numbers, hyphen/underscore-style separators.
