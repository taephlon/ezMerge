# ezMerge & Portage Integration

ezMerge is designed as a **transparent companion** rather than a layer that hides details. Every configuration change made by ezMerge is written in standard Portage format so that your system remains in a completely standard, self-documenting state.

## 🗃️ Where Configurations are Written

ezMerge isolates its modifications to specific files to avoid cluttering your hand-written configurations:

### 1. USE Flags (`package.use/ezmerge`)
Instead of modifying `/etc/portage/package.use` directly, ezMerge creates:
```path
/etc/portage/package.use/ezmerge
```
Entries are appended in the standard format:
```gentoo
category/package-name flag1 flag2 -flag3
```

### 2. Keyword Acceptances (`package.accept_keywords/ezmerge`)
For keyword-masked packages (e.g. `~amd64`), ezMerge appends to:
```path
/etc/portage/package.accept_keywords/ezmerge
```
Entries are written as:
```gentoo
category/package-name ~amd64
```

### 3. Mask Overrides (`package.unmask/ezmerge`)
For packages explicitly masked by profiles or package.mask, ezMerge writes to:
```path
/etc/portage/package.unmask/ezmerge
```

---

## ⏪ Rollback changes

If you ever wish to undo the configuration modifications made by ezMerge, you have two options:

1. **Via CLI**:
   Run:
   ```bash
   ezmerge undo
   ```
   This will automatically find the `/etc/portage/package.use/ezmerge` and accept_keywords files and safely remove them.

2. **Manually**:
   Since these are standard Portage configurations, you can inspect and delete them manually:
   ```bash
   rm /etc/portage/package.use/ezmerge
   rm /etc/portage/package.accept_keywords/ezmerge
   ```
