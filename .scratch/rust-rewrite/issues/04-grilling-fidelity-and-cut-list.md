# Grilling: what 'exact UI' means and what is dead weight
Type: grilling
Status: open
Blocked by:

## Question

Define the acceptance bar for 'replicate the original UI exactly' (pixel diff per screen, same layout and behavior, same QSS, same icons, same translations) and what is allowed to differ per OS (native dialogs, HiDPI, dark mode). Then walk the feature list (`DOCS/`, `src/`) and decide what is dead weight to cut: update manager, WinXP and 32-bit paths, X11 compositing hack in `src/platform/linux.cpp`, others found on review.
