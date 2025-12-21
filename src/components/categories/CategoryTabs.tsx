import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Category } from "../../types";

interface CategoryTabsProps {
  categories: Category[];
  selectedCategoryId: string;
  onSelectCategory: (categoryId: string) => void;
  onCategoriesChange: () => Promise<void>;
  openContextMenuId: string | null;
  onContextMenuChange: (categoryId: string | null) => void;
}

export default function CategoryTabs({
  categories,
  selectedCategoryId,
  onSelectCategory,
  onCategoriesChange,
  openContextMenuId,
  onContextMenuChange,
}: CategoryTabsProps) {
  const [isAdding, setIsAdding] = useState(false);
  const [newCategoryName, setNewCategoryName] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [editingCategoryId, setEditingCategoryId] = useState<string | null>(
    null
  );
  const [editingCategoryName, setEditingCategoryName] = useState("");

  // Sort categories by sort_order
  const sortedCategories = [...categories].sort(
    (a, b) => a.sort_order - b.sort_order
  );

  const handleAddCategory = async () => {
    if (!newCategoryName.trim()) return;

    try {
      setIsSubmitting(true);
      await invoke("add_category", {
        name: newCategoryName.trim(),
        icon: null,
      });
      await onCategoriesChange();
      setNewCategoryName("");
      setIsAdding(false);
    } catch (error) {
      console.error("Failed to add category:", error);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      handleAddCategory();
    } else if (e.key === "Escape") {
      setIsAdding(false);
      setNewCategoryName("");
    }
  };

  const handleEditCategory = (categoryId: string) => {
    const category = categories.find((c) => c.id === categoryId);
    if (!category) return;

    setEditingCategoryId(categoryId);
    setEditingCategoryName(category.name);
    onContextMenuChange(null);
  };

  const handleSaveEdit = async () => {
    if (!editingCategoryId || !editingCategoryName.trim()) return;

    try {
      await invoke("update_category", {
        categoryId: editingCategoryId,
        name: editingCategoryName.trim(),
        icon: null,
      });
      await onCategoriesChange();
      setEditingCategoryId(null);
      setEditingCategoryName("");
    } catch (error) {
      console.error("Failed to update category:", error);
      alert(`Failed to update category: ${error}`);
    }
  };

  const handleCancelEdit = () => {
    setEditingCategoryId(null);
    setEditingCategoryName("");
  };

  const handleDeleteCategory = async (categoryId: string) => {
    const category = categories.find((c) => c.id === categoryId);
    if (!category) return;

    if (
      !confirm(
        `Delete category "${category.name}"?\n\nWarning: All sounds in this category will also be deleted!`
      )
    ) {
      return;
    }

    try {
      await invoke("delete_category", { categoryId });
      await onCategoriesChange();
      onContextMenuChange(null);
    } catch (error) {
      console.error("Failed to delete category:", error);
      alert(`Failed to delete category: ${error}`);
    }
  };

  const handleContextMenu = (e: React.MouseEvent, categoryId: string) => {
    e.preventDefault();
    e.stopPropagation(); // Prevent document click listener
    onContextMenuChange(categoryId);
  };

  // Close context menu when clicking outside
  useEffect(() => {
    const handleClick = () => onContextMenuChange(null);
    if (openContextMenuId) {
      document.addEventListener("click", handleClick);
      return () => document.removeEventListener("click", handleClick);
    }
  }, [openContextMenuId, onContextMenuChange]);

  return (
    <div className="flex items-center gap-2 overflow-x-auto">
      {sortedCategories.map((category) => (
        <div key={category.id} className="relative">
          {editingCategoryId === category.id ? (
            // Edit mode
            <div className="flex items-center gap-2">
              <input
                type="text"
                value={editingCategoryName}
                onChange={(e) => setEditingCategoryName(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleSaveEdit();
                  if (e.key === "Escape") handleCancelEdit();
                }}
                className="px-3 py-2 bg-discord-darker border border-discord-primary rounded-lg
                         text-discord-text text-sm focus:outline-none"
                autoFocus
              />
              <button
                onClick={handleSaveEdit}
                className="px-3 py-2 bg-discord-primary hover:bg-discord-primary-hover
                         rounded-lg text-white text-sm transition-colors"
              >
                ✓
              </button>
              <button
                onClick={handleCancelEdit}
                className="px-3 py-2 bg-discord-dark hover:bg-discord-darker
                         rounded-lg text-discord-text-muted text-sm transition-colors"
              >
                ✕
              </button>
            </div>
          ) : (
            // Normal mode
            <>
              <button
                onClick={() => onSelectCategory(category.id)}
                onContextMenu={(e) => handleContextMenu(e, category.id)}
                className={`px-4 py-2 rounded-lg font-medium whitespace-nowrap transition-colors ${
                  selectedCategoryId === category.id
                    ? "bg-discord-primary text-white"
                    : "bg-discord-dark text-discord-text-muted hover:bg-discord-darker hover:text-discord-text"
                }`}
              >
                {category.icon && <span className="mr-2">{category.icon}</span>}
                {category.name}
              </button>

              {/* Context Menu - Must be above everything */}
              {openContextMenuId === category.id && (
                <div
                  className="fixed top-full left-0 mt-1 z-[9999] bg-discord-darker
                            border border-discord-dark rounded-lg shadow-xl py-1 min-w-32"
                  style={{
                    position: "fixed",
                    top: "160px",
                    left: "auto",
                  }}
                  onClick={(e) => e.stopPropagation()} // Prevent closing when clicking inside menu
                >
                  <button
                    onClick={() => handleEditCategory(category.id)}
                    className="w-full px-4 py-2 text-left text-sm text-discord-text
                             hover:bg-discord-primary hover:text-white transition-colors"
                  >
                    Edit Category
                  </button>
                  <button
                    onClick={() => handleDeleteCategory(category.id)}
                    className="w-full px-4 py-2 text-left text-sm text-discord-danger
                             hover:bg-discord-danger hover:text-white transition-colors"
                  >
                    Delete Category
                  </button>
                </div>
              )}
            </>
          )}
        </div>
      ))}

      {/* Add Category */}
      {isAdding ? (
        <div className="flex items-center gap-2">
          <input
            type="text"
            value={newCategoryName}
            onChange={(e) => setNewCategoryName(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Category name..."
            autoFocus
            disabled={isSubmitting}
            className="px-3 py-2 bg-discord-darker border border-discord-dark rounded-lg
                     text-discord-text text-sm focus:outline-none focus:ring-2
                     focus:ring-discord-primary disabled:opacity-50"
          />
          <button
            onClick={handleAddCategory}
            disabled={isSubmitting || !newCategoryName.trim()}
            className="px-3 py-2 bg-discord-success hover:bg-green-600
                     disabled:bg-gray-600 disabled:cursor-not-allowed rounded-lg
                     text-white text-sm font-medium transition-colors"
          >
            {isSubmitting ? "..." : "Add"}
          </button>
          <button
            onClick={() => {
              setIsAdding(false);
              setNewCategoryName("");
            }}
            disabled={isSubmitting}
            className="px-3 py-2 bg-discord-dark hover:bg-discord-darker rounded-lg
                     text-discord-text-muted text-sm transition-colors"
          >
            Cancel
          </button>
        </div>
      ) : (
        <button
          onClick={() => setIsAdding(true)}
          className="px-4 py-2 rounded-lg font-medium whitespace-nowrap transition-colors
                   bg-discord-dark text-discord-text-muted hover:bg-discord-darker
                   hover:text-discord-text border border-dashed border-discord-text-muted"
        >
          + Add Category
        </button>
      )}
    </div>
  );
}
