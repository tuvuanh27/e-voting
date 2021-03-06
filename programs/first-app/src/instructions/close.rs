use crate::errors::ErrorCode;
use crate::schema::*;
use anchor_lang::prelude::*;
use anchor_spl::{associated_token, token};

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, has_one = mint)]
    pub candidate: Account<'info, Candidate>,

    #[account(seeds = [b"treasurer", &candidate.key().to_bytes()], bump)]
    /// CHECK: Just a pure account
    pub treasurer: AccountInfo<'info>,
    pub mint: Box<Account<'info, token::Mint>>,

    #[account(mut, associated_token::mint = mint, associated_token::authority = authority)]
    pub candidate_account_token: Account<'info, token::TokenAccount>,

    // wallet account
    #[account(mut, close = authority, seeds=[b"ballot", &candidate.key().to_bytes(), &authority.key().to_bytes()], bump)]
    pub ballot: Account<'info, Ballot>,

    #[account(mut, associated_token::mint = mint, associated_token::authority = authority)]
    pub voter_token_account: Account<'info, token::TokenAccount>,

    // system
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, token::Token>,
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn exec(ctx: Context<Close>) -> Result<()> {
    let candidate = &mut ctx.accounts.candidate;
    let ballot = &mut ctx.accounts.ballot;

    let now = Clock::get().unwrap().unix_timestamp;
    if now < candidate.end_date {
        return err!(ErrorCode::NotEndedCandidate);
    }

    let seeds: &[&[&[u8]]] = &[&[
        "treasurer".as_ref(),
        &candidate.key().to_bytes(),
        &[*ctx.bumps.get("treasurer").unwrap()],
    ]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.candidate.to_account_info(),
        token::Transfer {
            from: ctx.accounts.candidate_account_token.to_account_info(),
            to: ctx.accounts.voter_token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        },
        seeds,
    );

    token::transfer(transfer_ctx, ballot.amount)?;
    ballot.amount = 0;

    Ok(())
}
