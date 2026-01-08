PYTHON ?= python3

SKILLS_INDEX_SCRIPT = skills/skill-indexer/scripts/build_skills_index.py
DOC_LINK_SCRIPT = skills/doc-linker/scripts/update_doc_links.py
SKILL_VALIDATE_SCRIPT = skills/skill-validator/scripts/validate_skills.py

.PHONY: skills-index skills-index-check doc-links doc-links-check validate-skills check

skills-index:
	$(PYTHON) $(SKILLS_INDEX_SCRIPT)

skills-index-check:
	$(PYTHON) $(SKILLS_INDEX_SCRIPT) --check

doc-links:
	$(PYTHON) $(DOC_LINK_SCRIPT) --write

doc-links-check:
	$(PYTHON) $(DOC_LINK_SCRIPT) --check

validate-skills:
	$(PYTHON) $(SKILL_VALIDATE_SCRIPT)

check: skills-index-check doc-links-check validate-skills
