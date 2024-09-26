#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(non_local_definitions)]

use ink::prelude::vec::Vec;
use ink::prelude::string::String;

#[ink::contract]
mod event_ticket_system {
    use super::*;
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct EventTicketSystem {
        tickets: Mapping<u64, Ticket>,
        ticket_owners: Mapping<AccountId, Vec<u64>>,
        marketplace_listings: Mapping<u64, Balance>,
        next_ticket_id: u64,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Ticket {
        id: u64,
        event_name: String,
        expiration_date: u64,
        owner: AccountId,
    }

    #[ink(event)]
    pub struct TicketIssued {
        #[ink(topic)]
        ticket_id: u64,
        owner: AccountId,
    }

    #[ink(event)]
    pub struct TicketTransferred {
        #[ink(topic)]
        ticket_id: u64,
        from: AccountId,
        to: AccountId,
    }

    #[ink(event)]
    pub struct TicketListed {
        #[ink(topic)]
        ticket_id: u64,
        price: Balance,
    }

    #[ink(event)]
    pub struct TicketSold {
        #[ink(topic)]
        ticket_id: u64,
        buyer: AccountId,
        price: Balance,
    }

    impl Default for EventTicketSystem {
        fn default() -> Self {
            Self::new()
        }
    }

    impl EventTicketSystem {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                tickets: Mapping::default(),
                ticket_owners: Mapping::default(),
                marketplace_listings: Mapping::default(),
                next_ticket_id: 0,
            }
        }

        #[ink(message)]
        pub fn issue_ticket(&mut self, event_name: String, expiration_date: u64) -> u64 {
            let ticket_id = self.next_ticket_id;
            self.next_ticket_id = self.next_ticket_id.saturating_add(1);

            let ticket = Ticket {
                id: ticket_id,
                event_name,
                expiration_date,
                owner: Self::env().account_id(),
            };

            self.tickets.insert(ticket_id, &ticket);
            let mut owner_tickets = self.ticket_owners.get(Self::env().account_id()).unwrap_or_default();
            owner_tickets.push(ticket_id);
            self.ticket_owners.insert(Self::env().account_id(), &owner_tickets);

            self.env().emit_event(TicketIssued {
                ticket_id,
                owner: Self::env().account_id(),
            });

            ticket_id
        }

        #[ink(message)]
        pub fn transfer_ticket(&mut self, ticket_id: u64, to: AccountId) -> bool {
            let caller = Self::env().account_id();

            if let Some(mut ticket) = self.tickets.get(ticket_id) {
                if ticket.owner != caller {
                    return false;
                }

                let from = ticket.owner;
                ticket.owner = to;
                self.tickets.insert(ticket_id, &ticket);
                
                let mut from_tickets = self.ticket_owners.get(from).unwrap_or_default();
                from_tickets.retain(|&id| id != ticket_id);
                self.ticket_owners.insert(from, &from_tickets);

                let mut to_tickets = self.ticket_owners.get(to).unwrap_or_default();
                to_tickets.push(ticket_id);
                self.ticket_owners.insert(to, &to_tickets);

                self.env().emit_event(TicketTransferred {
                    ticket_id,
                    from,
                    to,
                });

                true
            } else {
                false
            }
        }

        #[ink(message)]
        pub fn list_ticket(&mut self, ticket_id: u64, price: Balance) -> bool {
            let caller = Self::env().account_id();

            if let Some(ticket) = self.tickets.get(ticket_id) {
                if ticket.owner != caller {
                    return false;
                }

                self.marketplace_listings.insert(ticket_id, &price);

                self.env().emit_event(TicketListed {
                    ticket_id,
                    price,
                });

                true
            } else {
                false
            }
        }

        #[ink(message, payable)]
        pub fn buy_ticket(&mut self, ticket_id: u64) -> bool {
            let caller = Self::env().account_id();
            let payment = Self::env().transferred_value();

            if let Some(price) = self.marketplace_listings.get(ticket_id) {
                if payment != price {
                    return false;
                }

                if let Some(mut ticket) = self.tickets.get(ticket_id) {
                    let previous_owner = ticket.owner;
                    ticket.owner = caller;
                    self.tickets.insert(ticket_id, &ticket);

                    let mut prev_owner_tickets = self.ticket_owners.get(previous_owner).unwrap_or_default();
                    prev_owner_tickets.retain(|&id| id != ticket_id);
                    self.ticket_owners.insert(previous_owner, &prev_owner_tickets);

                    let mut new_owner_tickets = self.ticket_owners.get(caller).unwrap_or_default();
                    new_owner_tickets.push(ticket_id);
                    self.ticket_owners.insert(caller, &new_owner_tickets);

                    self.marketplace_listings.remove(ticket_id);

                    if self.env().transfer(previous_owner, payment).is_err() {
                        panic!("Transfer to previous owner failed");
                    }

                    self.env().emit_event(TicketSold {
                        ticket_id,
                        buyer: caller,
                        price: payment,
                    });

                    true
                } else {
                    false
                }
            } else {
                false
            }
        }

        #[ink(message)]
        pub fn verify_ticket(&self, ticket_id: u64) -> bool {
            self.tickets.contains(ticket_id)
        }

        #[ink(message)]
        pub fn get_ticket(&self, ticket_id: u64) -> Option<Ticket> {
            self.tickets.get(ticket_id)
        }

        #[ink(message)]
        pub fn get_user_tickets(&self, user: AccountId) -> Vec<u64> {
            self.ticket_owners.get(user).unwrap_or_default()
        }

        #[ink(message)]
        pub fn get_ticket_price(&self, ticket_id: u64) -> Option<Balance> {
            self.marketplace_listings.get(ticket_id)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn issue_ticket_works() {
            let mut contract = EventTicketSystem::new();
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            
            let ticket_id = contract.issue_ticket("Concert".into(), 1625097600);
            assert_eq!(ticket_id, 0);
            assert!(contract.verify_ticket(ticket_id));
            
            let issued_ticket = contract.get_ticket(ticket_id).unwrap();
            assert_eq!(issued_ticket.owner, accounts.alice);
        }

        #[ink::test]
        fn transfer_ticket_works() {
            let mut contract = EventTicketSystem::new();
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            
            let ticket_id = contract.issue_ticket("Concert".into(), 1625097600);
            assert!(contract.transfer_ticket(ticket_id, accounts.bob));
            
            let transferred_ticket = contract.get_ticket(ticket_id).unwrap();
            assert_eq!(transferred_ticket.owner, accounts.bob);
        }

        #[ink::test]
        fn marketplace_works() {
            let mut contract = EventTicketSystem::new();
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            
            let ticket_id = contract.issue_ticket("Concert".into(), 1625097600);
            assert!(contract.list_ticket(ticket_id, 1000));
            
            assert_eq!(contract.get_ticket_price(ticket_id), Some(1000));
            
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(1000);
            
            assert!(contract.buy_ticket(ticket_id));
            
            let bought_ticket = contract.get_ticket(ticket_id).unwrap();
            assert_eq!(bought_ticket.owner, accounts.bob);
        }
    }
}