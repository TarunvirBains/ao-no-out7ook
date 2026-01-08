---
description: How to decompose a User Story into granular Tasks using the CLI
---

1. **Read the Story Context**
   Fetch the details of the parent User Story to understand requirements.
   ```bash
   ano7 show <STORY_ID>
   ```

2. **Workspace Analysis & Context Gathering**
   Now that you understand the requirements, identify the relevant code.
   *   **Analyze:** Explore the directory structure to find components/files related to the story.
   *   **Prompt User:** "I've read the story. Please provide any specific files or design docs if available."
   *   **Read:** Read the gathered files to ensure the implementation plan is technically sound.

3. **Plan the Tasks (Generate Markdown)**
   Create a markdown file (e.g., `plan.md`) defining the child tasks. 
   Use ID `0` to indicate new items. Set the parent ID to the User Story ID.

   *Example `plan.md`:*
   ```markdown
   #### Task: Implement Login UI (#0)
   **State:** New | **Parent:** #<STORY_ID> | **Effort:** 2h
   Create the React component for the login form.
   
   #### Task: Integrate Auth API (#0)
   **State:** New | **Parent:** #<STORY_ID> | **Effort:** 3h
   Connect the form to the backend auth endpoint.
   ```

4. **Validate the Plan**
   Check for hierarchy or syntax errors before executing.
   ```bash
   ano7 import plan.md --validate
   ```

5. **Execute Breakdown (Create Items)**
   Import the markdown to create the work items in DevOps.
   ```bash
   ano7 import plan.md
   ```

6. **Schedule Work (Optional)**
   After import, use the Output or `ano7 list` to find the new IDs, then schedule them.
   ```bash
   ano7 calendar schedule <NEW_TASK_ID> --duration 120 --title "Login UI"
   ano7 calendar schedule <OTHER_TASK_ID> --duration 180 --title "Auth API"
   ```
