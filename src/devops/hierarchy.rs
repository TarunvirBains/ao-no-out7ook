use crate::devops::client::DevOpsClient;
use crate::devops::models::WorkItem;
use anyhow::Result;
use std::fmt;
use termtree::Tree;

pub struct HierarchyNode {
    pub item: WorkItem,
    pub children: Vec<HierarchyNode>,
}

impl fmt::Display for HierarchyNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let title = self.item.get_title().unwrap_or("No Title");
        let state = self.item.get_state().unwrap_or("Unknown");
        let id = self.item.id;
        write!(f, "#{} {} [{}]", id, title, state)
    }
}

pub fn build_tree(client: &DevOpsClient, root_id: u32, depth: u8) -> Result<HierarchyNode> {
    let root = client.get_work_item(root_id)?;

    if depth == 0 {
        return Ok(HierarchyNode {
            item: root,
            children: Vec::new(),
        });
    }

    let mut children = Vec::new();
    if let Some(relations) = &root.relations {
        let child_ids: Vec<u32> = relations
            .iter()
            .filter(|r| r.rel == "System.LinkTypes.Hierarchy-Forward")
            .filter_map(|r| {
                // "url": "https://.../_apis/wit/workItems/123"
                r.url.split('/').next_back().and_then(|s| s.parse().ok())
            })
            .collect();

        if !child_ids.is_empty() {
            // Optimization: Batch fetch immediate children
            let child_items = client.get_work_items_batch(&child_ids)?;

            for child_item in child_items {
                // For each child, recurse?
                // If we batch fetched, we have the item. But to get ITS children, we need its relations.
                // The batch fetch usually returns relations if $expand=all is set (which we did).

                // So we can convert WorkItem to HierarchyNode recursively?
                // But wait, `build_tree` calls `get_work_item`.
                // We should refactor to `build_tree_from_item`.

                let node = build_tree_recursive(client, child_item, depth - 1)?;
                children.push(node);
            }
        }
    }

    Ok(HierarchyNode {
        item: root,
        children,
    })
}

fn build_tree_recursive(client: &DevOpsClient, item: WorkItem, depth: u8) -> Result<HierarchyNode> {
    if depth == 0 {
        return Ok(HierarchyNode {
            item,
            children: Vec::new(),
        });
    }

    let mut children = Vec::new();
    if let Some(relations) = &item.relations {
        let child_ids: Vec<u32> = relations
            .iter()
            .filter(|r| r.rel == "System.LinkTypes.Hierarchy-Forward")
            .filter_map(|r| r.url.split('/').next_back().and_then(|s| s.parse().ok()))
            .collect();

        if !child_ids.is_empty() {
            let child_items = client.get_work_items_batch(&child_ids)?;
            for child_item in child_items {
                let node = build_tree_recursive(client, child_item, depth - 1)?;
                children.push(node);
            }
        }
    }

    Ok(HierarchyNode { item, children })
}

pub fn print_tree(node: &HierarchyNode) {
    let tree = build_termtree(node);
    println!("{}", tree);
}

fn build_termtree(node: &HierarchyNode) -> Tree<String> {
    let label = node.to_string();
    let mut tree = Tree::new(label);

    for child in &node.children {
        tree.push(build_termtree(child));
    }

    tree
}
