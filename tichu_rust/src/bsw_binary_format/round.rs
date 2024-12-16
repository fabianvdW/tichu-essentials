use bitcode::{Decode, Encode};
use crate::bsw_binary_format::binary_format_constants::{PLAYER_0, PLAYER_1, PLAYER_2, PLAYER_3};
use crate::bsw_binary_format::player_round_hand::PlayerRoundHand;
use crate::tichu_hand::MASK_ALL;

#[derive(Encode, Decode, Default)]
pub struct Round {
    pub player_rounds: [PlayerRoundHand; 4],
}
impl Round {
    pub fn integrity_check(&self) {
        for i in 0..4 {
            self.player_rounds[i].integrity_check();
        }
        let (p0, p1, p2, p3) = (
            self.player_rounds.get(0).unwrap(),
            self.player_rounds.get(1).unwrap(),
            self.player_rounds.get(2).unwrap(),
            self.player_rounds.get(3).unwrap(),
        );
        //Check all cards are distributed. Child checks ensures its 14 each.
        assert_eq!(
            p0.first_14 | p1.first_14 | p2.first_14 | p3.first_14,
            MASK_ALL
        );
        //Check exchange cards
        assert_eq!(p0.right_out_exchange_card(), p1.left_in_exchange_card());
        assert_eq!(p1.right_out_exchange_card(), p2.left_in_exchange_card());
        assert_eq!(p2.right_out_exchange_card(), p3.left_in_exchange_card());
        assert_eq!(p3.right_out_exchange_card(), p0.left_in_exchange_card());

        assert_eq!(p0.left_out_exchange_card(), p3.right_in_exchange_card());
        assert_eq!(p3.left_out_exchange_card(), p2.right_in_exchange_card());
        assert_eq!(p2.left_out_exchange_card(), p1.right_in_exchange_card());
        assert_eq!(p1.left_out_exchange_card(), p0.right_in_exchange_card());

        assert_eq!(p0.partner_out_exchange_card(), p2.partner_in_exchange_card());
        assert_eq!(p2.partner_out_exchange_card(), p0.partner_in_exchange_card());
        assert_eq!(p1.partner_out_exchange_card(), p3.partner_in_exchange_card());
        assert_eq!(p3.partner_out_exchange_card(), p1.partner_in_exchange_card());

        //Check that all other fields agree on the same values
        assert_eq!((p0.extras >> 36) & 0xFF, (p1.extras >> 36) & 0xFF);
        assert_eq!((p1.extras >> 36) & 0xFF, (p2.extras >> 36) & 0xFF);
        assert_eq!((p2.extras >> 36) & 0xFF, (p3.extras >> 36) & 0xFF);

        assert_eq!(p0.extras >> 46, p1.extras >> 46);
        assert_eq!(p1.extras >> 46, p2.extras >> 46);
        assert_eq!(p2.extras >> 46, p3.extras >> 46);

        assert_eq!(p0.player_id(), PLAYER_0);
        assert_eq!(p1.player_id(), PLAYER_1);
        assert_eq!(p2.player_id(), PLAYER_2);
        assert_eq!(p3.player_id(), PLAYER_3);

        assert_eq!(p0.extras >> 54, p1.extras >> 54);
        assert_eq!(p1.extras >> 54, p2.extras >> 54);
        assert_eq!(p2.extras >> 54, p3.extras >> 54);
    }
}