# Gates: unified README structure on main and develop

OWNS: README.md, README_ko.md, README_ja.md, tools/test_documentation.py, GATES.md

Scope: replace chronological development logs with one coherent three-language README structure, preserve essential operational facts, and publish the result to both main and develop with an explicit develop warning.

- [x] G0: this ledger states executable outcomes that can fail
  CHECK: node C:\Users\kjh\.codex\skills\unlazy\scripts\gate-lint.mjs GATES.md
  EXPECT: LINT OK
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=7f36783b8116/33 entries; output=WARN  G4: no CHECK, so this outcome is judged by hand and its evidence is only as good as the reader  [manual-gate] | LINT OK (1 warning(s))

- [x] G1: all three README editions put essentials first, incomplete work in the middle, and implemented features below it
  CHECK: python -B -m unittest tools.test_documentation.DocumentationTests.test_readme_information_architecture && echo README architecture verification passed
  EXPECT: README architecture verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=7f36783b8116/33 entries; output=Ran 1 test in 0.000s | OK

- [x] G2: the complete documentation contract preserves build, recovery, layout, OS, ordering, and hardware-validation guidance
  CHECK: python -B tools/test_documentation.py && echo README documentation contract passed
  EXPECT: README documentation contract passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=7f36783b8116/33 entries; output=Ran 10 tests in 0.006s | OK

- [x] G3: the rewritten READMEs contain no chronological P-stage sections and retain one coherent feature-based narrative
  CHECK: python -B -m unittest tools.test_documentation.DocumentationTests.test_readme_has_no_chronological_development_log && echo README narrative verification passed
  EXPECT: README narrative verification passed
  EVIDENCE: exit=0; shell=C:\WINDOWS\system32\cmd.exe; cwd=D:\study\nocfree\NocFree-and-rust; path=7f36783b8116/33 entries; output=Ran 1 test in 0.000s | OK

- [ ] G4: the final README structure is published to both remote branches and develop remains explicitly hardware-unverified
  EVIDENCE: pending
