use crate::models::gamification::{BuyFreezeTicketResponse, UserStats};
use crate::repositories::gamification_repo::GamificationRepo;
use axum::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

pub struct GamificationService;

impl GamificationService {
    pub fn get_tier_by_exp(exp: i32) -> (i32, &'static str) {
        match exp {
            e if e >= 254262 => (10, "Zenith"),
            e if e >= 101665 => (9, "Apex"),
            e if e >= 40626 => (8, "Nova"),
            e if e >= 16210 => (7, "Prime"),
            e if e >= 6443 => (6, "Mastermind"),
            e if e >= 2537 => (5, "Vanguard"),
            e if e >= 975 => (4, "Architect"),
            e if e >= 350 => (3, "Momentum"),
            e if e >= 100 => (2, "Catalyst"),
            _ => (1, "Spark"),
        }
    }

    pub async fn add_reward(
        pool: &PgPool,
        user_id: Uuid,
        exp_to_add: i32,
        coins_to_add: i32,
    ) -> Result<UserStats, (StatusCode, String)> {
        let current_stats = GamificationRepo::get_stats(pool, user_id).await?;

        let new_exp = current_stats.exp + exp_to_add;
        let mut new_coins = current_stats.coins + coins_to_add;

        let (current_tier_level, _) = Self::get_tier_by_exp(current_stats.exp);
        let (new_tier_level, _) = Self::get_tier_by_exp(new_exp);

        if new_tier_level > current_tier_level {
            new_coins += new_tier_level * 50; // Bonus coins on level up
        }

        GamificationRepo::update_stats(
            pool,
            user_id,
            new_exp,
            new_coins,
            new_tier_level,
            current_stats.freeze_tickets,
        )
        .await
    }

    pub async fn buy_freeze_ticket(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<BuyFreezeTicketResponse, (StatusCode, String)> {
        let stats = GamificationRepo::get_stats(pool, user_id).await?;

        if stats.coins < 200 {
            return Ok(BuyFreezeTicketResponse {
                success: false,
                message: "Not enough coins. Need 200 coins.".to_string(),
                remaining_coins: stats.coins,
                total_tickets: stats.freeze_tickets,
            });
        }

        let updated_stats = GamificationRepo::update_stats(
            pool,
            user_id,
            stats.exp,
            stats.coins - 200,
            stats.tier,
            stats.freeze_tickets + 1,
        )
        .await?;

        Ok(BuyFreezeTicketResponse {
            success: true,
            message: "Freeze Ticket purchased successfully!".to_string(),
            remaining_coins: updated_stats.coins,
            total_tickets: updated_stats.freeze_tickets,
        })
    }
}
