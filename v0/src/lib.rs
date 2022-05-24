use assert_panic::assert_panic;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{assert_one_yocto, assert_self, env, near_bindgen, BorshStorageKey};

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
  Orders,
}

#[derive(BorshStorageKey, BorshSerialize, BorshDeserialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub enum OrderStatus {
  Pending,
  Scheduled,
  InProgress,
  Completed,
  Cancelled,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Order {
  order_id: String,
  status: OrderStatus,
  description: String,
}

#[near_bindgen]
impl KiranaOrderStatus {
  #[payable]
  pub fn create_order(&mut self, _order: Order) -> String {
    assert_self();
    assert_one_yocto();
    let order = self.orders.get(&_order.order_id);
    match order {
      Some(_) => env::panic(String::from("Order already exists").as_bytes()),
      None => {}
    }
    self.orders.insert(&_order.order_id, &_order);
    String::from("OK")
  }
  pub fn get_order(self, order_id: String) -> Order {
    self.orders.get(&order_id).expect("Order does not exist")
  }

  #[payable]
  pub fn update_order(&mut self, _order: Order) -> String {
    assert_self();
    assert_one_yocto();
    let mut order = self.orders.get(&_order.order_id).expect("Order does not exist");
    order.description = _order.description;
    order.status = _order.status;
    self.orders.insert(&_order.order_id, &order);
    String::from("OK")
  }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct KiranaOrderStatus {
  orders: UnorderedMap<String, Order>,
}

impl Default for KiranaOrderStatus {
  fn default() -> Self {
    Self {
      orders: UnorderedMap::new(StorageKey::Orders),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use near_sdk::MockedBlockchain;
  use near_sdk::{testing_env, VMContext};

  // mock the context for testing, notice "signer_account_id" that was accessed above from env::
  fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
    VMContext {
      current_account_id: "v0-kos-kirananetwork.near".to_string(),
      signer_account_id: "v0-kos-kirananetwork.near".to_string(),
      signer_account_pk: vec![0, 1, 2],
      predecessor_account_id: "v0-kos-kirananetwork.near".to_string(),
      input,
      block_index: 0,
      block_timestamp: 0,
      account_balance: 0,
      account_locked_balance: 0,
      storage_usage: 0,
      attached_deposit: 0,
      prepaid_gas: 10u64.pow(18),
      random_seed: vec![0, 1, 2],
      is_view,
      output_data_receivers: vec![],
      epoch_height: 19,
    }
  }

  #[test]
  fn only_contract_predecessor_account_can_call_this_contract() {
    // create order
    assert_panic!({
      let mut context = get_context(vec![], false);
      context.predecessor_account_id = String::from("other.test.near");
      testing_env!(context);
      let mut contract = KiranaOrderStatus::default();
      contract.create_order(Order { 
        order_id: String::from("order_id"),
        description: String::from("desc"),
        status: OrderStatus::Pending
      });
      // contract.create_order(Order { });
    }, String, starts with "assertion failed:")
  }

  #[test]
  fn create_order_requires_deposit() {
    // create order
    assert_panic!({
      let context = get_context(vec![], false);
      testing_env!(context);
      let mut contract = KiranaOrderStatus::default();
      contract.create_order(Order { 
        order_id: String::from("order_id"),
        description: String::from("desc"),
        status: OrderStatus::Pending
      });
      // contract.create_order(Order { });
    }, String, starts with "assertion failed:")
  }

  #[test]
  fn should_create_order() {
    // create order
    let mut context = get_context(vec![], false);
    context.attached_deposit = 1;
    testing_env!(context);
    let mut contract = KiranaOrderStatus::default();
    assert_eq!(contract.orders.len(), 0);
    let order_created = contract.create_order(Order {
      order_id: String::from("order_id"),
      description: String::from("desc"),
      status: OrderStatus::Pending,
    });
    assert_eq!(String::from("OK"), order_created);
    assert_eq!(contract.orders.len(), 1);
  }

  #[test]
  fn create_order_fails_if_order_already_exists() {
    // create order
    assert_panic!({
      let mut context = get_context(vec![], false);
      context.attached_deposit = 1;
      testing_env!(context);
      let mut contract = KiranaOrderStatus::default();
      let order = Order{ 
        order_id: String::from("order_id"),
        description: String::from("desc"),
        status: OrderStatus::Pending
      };
      contract.orders.insert(&String::from("order_id"), &order);
      contract.create_order(order);
      // contract.create_order(Order { });
    }, String, contains "Order already exists")
  }

  // test: get_order
  #[test]
  fn should_panic_on_get_order_if_not_exists() {
    assert_panic!({
      let mut context = get_context(vec![], false);
      context.attached_deposit = 1;
      testing_env!(context);
      let contract = KiranaOrderStatus::default();
      contract.get_order(String::from("order_id"));
    }, String, contains "Order does not exist")
  }

  #[test]
  fn should_return_order() {
    let mut context = get_context(vec![], false);
    context.attached_deposit = 1;
    testing_env!(context);
    let mut contract = KiranaOrderStatus::default();
    contract.orders.insert(
      &String::from("order_id"),
      &Order {
        description: String::from("description"),
        order_id: String::from("order_id"),
        status: OrderStatus::Scheduled,
      },
    );
    let order = contract.get_order(String::from("order_id"));
    assert_eq!(order.order_id, "order_id");
  }

  // test: update_order requires predecessor check
  // test: update_order requires 1 yocto to be deposited
  // test: should not allow changing the order_id for existing order

  #[test]
  fn should_panic_update_order_if_not_predecessor() {
    // create order
    assert_panic!({
      let mut context = get_context(vec![], false);
      context.predecessor_account_id = String::from("other.near");
      testing_env!(context);
      let mut contract = KiranaOrderStatus::default();
      contract.update_order(Order { 
        order_id: String::from("order_id"),
        description: String::from("desc"),
        status: OrderStatus::Pending
      });
      // contract.create_order(Order { });
    }, String, starts with "assertion failed:")
  }

  #[test]
  fn should_panic_update_order_if_no_deposit() {
    // create order
    assert_panic!({
      let mut context = get_context(vec![], false);
      context.attached_deposit = 0;
      testing_env!(context);
      let mut contract = KiranaOrderStatus::default();
      contract.update_order(Order { 
        order_id: String::from("order_id"),
        description: String::from("desc"),
        status: OrderStatus::Pending
      });
      // contract.create_order(Order { });
    }, String, starts with "assertion failed:")
  }

  #[test]
  fn should_panic_update_order_if_no_order() {
    // create order
    assert_panic!({
      let mut context = get_context(vec![], false);
      context.attached_deposit = 1;
      testing_env!(context);
      let mut contract = KiranaOrderStatus::default();
      contract.update_order(Order { 
        order_id: String::from("order_id"),
        description: String::from("desc"),
        status: OrderStatus::Pending
      });
    }, String, contains "Order does not exist")
  }

  #[test]
  fn should_update_order() {
    let mut context = get_context(vec![], false);
      context.attached_deposit = 1;
      testing_env!(context);
      let mut contract = KiranaOrderStatus::default();
      contract.orders.insert(
        &String::from("order_id"),
        &Order {
          description: String::from("description"),
          order_id: String::from("order_id"),
          status: OrderStatus::Scheduled,
        },
      );
      let result = contract.update_order(Order { 
        order_id: String::from("order_id"),
        description: String::from("desc"),
        status: OrderStatus::Pending
      });
      assert_eq!(result, String::from("OK"));
      let order = contract.get_order(String::from("order_id"));
      assert_eq!(order.description, String::from("desc"));
  }
}
