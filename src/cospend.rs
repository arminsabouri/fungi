use crate::transaction::Outpoint;
use crate::wallet::WalletId;
use bitcoin::Amount;

#[derive(Debug, Clone)]
pub(crate) struct UtxoWithAmount {
    pub(crate) outpoint: Outpoint,
    pub(crate) amount: Amount,
}

/// A UTXO from the order book
#[derive(Debug, Clone)]
pub(crate) struct OrderBookEntry {
    pub(crate) utxo: UtxoWithAmount,
    pub(crate) owner: WalletId,
}

/// A candidate co-spend: a specific subset of taker UTXOs combined with a specific subset
/// of order book UTXOs.
pub(crate) struct CospendCandidate {
    pub(crate) taker_inputs: Vec<UtxoWithAmount>,
    pub(crate) ob_entries: Vec<OrderBookEntry>,
}

fn amount_distance(a: Amount, b: Amount) -> u64 {
    a.to_sat().abs_diff(b.to_sat())
}

/// Build cospend candidates by walking all available order-book entries for each taker UTXO.
/// For each taker UTXO, order-book entries are visited from closest amount to farthest amount,
/// preferring magnitude-matched pairings and collecting every possible cospend pair.
pub(crate) fn generate_candidates(
    order_book: &[OrderBookEntry],
    utxos: &[UtxoWithAmount],
) -> Vec<CospendCandidate> {
    let mut candidates = Vec::with_capacity(utxos.len() * order_book.len());

    for taker in utxos {
        let mut ordered_entries: Vec<&OrderBookEntry> = order_book.iter().collect();
        ordered_entries.sort_unstable_by(|a, b| {
            let dist_a = amount_distance(a.utxo.amount, taker.amount);
            let dist_b = amount_distance(b.utxo.amount, taker.amount);
            dist_a
                .cmp(&dist_b)
                .then_with(|| a.utxo.outpoint.txid.0.cmp(&b.utxo.outpoint.txid.0))
                .then_with(|| a.utxo.outpoint.index.cmp(&b.utxo.outpoint.index))
        });

        for entry in ordered_entries {
            let taker_inputs = vec![taker.clone()];
            let ob_entries = vec![entry.clone()];
            candidates.push(CospendCandidate {
                taker_inputs,
                ob_entries,
            });
        }
    }

    candidates
}

/// Among the `top_k` candidates by score, count how often each order book entry appears.
/// Returns all entries sorted descending by frequency (zero-frequency entries included).
pub(crate) fn find_common_entries(
    candidates: &[CospendCandidate],
    order_book: &[OrderBookEntry],
) -> Vec<(OrderBookEntry, f64)> {
    let denom = if candidates.is_empty() {
        1.0
    } else {
        candidates.len() as f64
    };

    let mut counts = vec![0usize; order_book.len()];
    for candidate in candidates {
        for entry in &candidate.ob_entries {
            if let Some(idx) = order_book
                .iter()
                .position(|e| e.utxo.outpoint == entry.utxo.outpoint)
            {
                counts[idx] += 1;
            }
        }
    }

    let mut ranked: Vec<(OrderBookEntry, f64)> = order_book
        .iter()
        .zip(&counts)
        .map(|(entry, &count)| (entry.clone(), count as f64 / denom))
        .collect();
    ranked.sort_unstable_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.utxo.outpoint.txid.0.cmp(&b.0.utxo.outpoint.txid.0))
            .then_with(|| a.0.utxo.outpoint.index.cmp(&b.0.utxo.outpoint.index))
    });
    ranked
}

/// Among the `top_k` candidates by score, count how often each taker UTXO appears.
/// Returns all UTXOs sorted descending by frequency (zero-frequency UTXOs included).
pub(crate) fn find_common_taker_utxos(
    candidates: &[CospendCandidate],
    utxos: &[UtxoWithAmount],
) -> Vec<(UtxoWithAmount, f64)> {
    let denom = if candidates.is_empty() {
        1.0
    } else {
        candidates.len() as f64
    };

    let mut counts = vec![0usize; utxos.len()];
    for candidate in candidates {
        for utxo in &candidate.taker_inputs {
            if let Some(idx) = utxos.iter().position(|u| u.outpoint == utxo.outpoint) {
                counts[idx] += 1;
            }
        }
    }

    let mut ranked: Vec<(UtxoWithAmount, f64)> = utxos
        .iter()
        .zip(&counts)
        .map(|(utxo, &count)| (utxo.clone(), count as f64 / denom))
        .collect();
    ranked.sort_unstable_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.outpoint.txid.0.cmp(&b.0.outpoint.txid.0))
            .then_with(|| a.0.outpoint.index.cmp(&b.0.outpoint.index))
    });
    ranked
}
