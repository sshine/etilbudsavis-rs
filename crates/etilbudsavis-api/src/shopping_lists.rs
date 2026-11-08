//! Shopping lists and their items.

use serde_json::{Value, json};

use crate::client::Client;
use crate::error::Result;
use crate::types::{
    CategorizedItem, CreatedShoppingList, ShoppingList, ShoppingListItem, ShoppingListItemDraft,
    ShoppingListItemUpdate,
};

impl Client {
    /// Lists owned by or shared with the current account.
    pub async fn shopping_lists(&self) -> Result<Vec<ShoppingList>> {
        self.data("shoppingLists", Value::Null).await
    }

    /// Items on one list.
    pub async fn shopping_list_items(&self, list_id: i64) -> Result<Vec<ShoppingListItem>> {
        self.data("shoppingListItems", json!({ "shoppingListId": list_id }))
            .await
    }

    /// Creates an empty list.
    pub async fn create_shopping_list(&self, name: &str) -> Result<CreatedShoppingList> {
        self.call_api("shopping-list-create-list", json!({ "name": name }))
            .await
    }

    /// Renames a list.
    pub async fn rename_shopping_list(&self, list_id: i64, new_name: &str) -> Result<()> {
        self.call_api_unit(
            "shopping-list-rename-list",
            json!({ "id": list_id, "newName": new_name }),
        )
        .await
    }

    /// Deletes a list for everyone.
    pub async fn delete_shopping_list(&self, list_id: i64) -> Result<()> {
        self.call_api_unit("shopping-list-delete-list", json!({ "id": list_id }))
            .await
    }

    /// Leaves a list shared by someone else.
    pub async fn leave_shopping_list(&self, list_id: i64) -> Result<()> {
        self.call_api_unit("shopping-list-leave-list", json!({ "id": list_id }))
            .await
    }

    /// Removes every collaborator from a list.
    pub async fn remove_shopping_list_collaborators(&self, list_id: i64) -> Result<()> {
        self.call_api_unit(
            "shopping-list-remove-collaborators",
            json!({ "id": list_id }),
        )
        .await
    }

    /// Accepts a share link and returns the list id.
    pub async fn accept_shopping_list_invitation(
        &self,
        share_token: &str,
    ) -> Result<CreatedShoppingList> {
        self.call_api(
            "shopping-list-accept-invitation",
            json!({ "shareToken": share_token }),
        )
        .await
    }

    /// Adds one item.
    pub async fn add_shopping_list_item(
        &self,
        list_id: i64,
        item: &ShoppingListItemDraft,
    ) -> Result<()> {
        self.call_api_unit(
            "shopping-list-add-item",
            json!({ "shoppingListId": list_id, "item": item }),
        )
        .await
    }

    /// Adds many items in one request.
    pub async fn add_shopping_list_items(
        &self,
        list_id: i64,
        items: &[ShoppingListItemDraft],
    ) -> Result<()> {
        self.call_api_unit(
            "shopping-list-bulk-add-items",
            json!({ "shoppingListId": list_id, "items": items }),
        )
        .await
    }

    /// Updates one item.
    pub async fn update_shopping_list_item(
        &self,
        list_id: i64,
        item: &ShoppingListItemUpdate,
    ) -> Result<()> {
        self.call_api_unit(
            "shopping-list-update-item",
            json!({ "shoppingListId": list_id, "item": item }),
        )
        .await
    }

    /// Updates many items in one request.
    pub async fn update_shopping_list_items(
        &self,
        list_id: i64,
        items: &[ShoppingListItemUpdate],
    ) -> Result<()> {
        self.call_api_unit(
            "shopping-list-bulk-update-items",
            json!({ "shoppingListId": list_id, "items": items }),
        )
        .await
    }

    /// Removes one item by its client id.
    pub async fn remove_shopping_list_item(&self, list_id: i64, item_id: &str) -> Result<()> {
        self.call_api_unit(
            "shopping-list-remove-item",
            json!({ "shoppingListId": list_id, "itemId": item_id }),
        )
        .await
    }

    /// Removes many items by their client ids.
    pub async fn remove_shopping_list_items(
        &self,
        list_id: i64,
        item_ids: &[String],
    ) -> Result<()> {
        self.call_api_unit(
            "shopping-list-bulk-remove-items",
            json!({ "shoppingListId": list_id, "itemIds": item_ids }),
        )
        .await
    }

    /// Asks the service to assign a department to a free-text item.
    pub async fn auto_categorize_shopping_list_item(
        &self,
        list_id: i64,
        client_id: &str,
    ) -> Result<CategorizedItem> {
        self.call_api(
            "shopping-list-auto-categorize-item",
            json!({ "shoppingListId": list_id, "clientId": client_id }),
        )
        .await
    }
}
