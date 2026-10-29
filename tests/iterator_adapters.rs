use mime_infer::{Iter, Mime, from_ext};

fn expected_values(ext: &str) -> Vec<Mime> {
    from_ext(ext)
        .iter_raw()
        .map(|raw| raw.parse().unwrap())
        .collect()
}

fn assert_exhausted(iter: &mut Iter) {
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
    assert_eq!(iter.nth(usize::MAX), None);
    assert_eq!(iter.nth_back(usize::MAX), None);
    assert_eq!(iter.clone().count(), 0);
    assert_eq!(iter.clone().last(), None);
}

#[test]
#[allow(clippy::iter_nth_zero)] // Exercise nth(0) alongside other adapter methods.
fn mixed_adapters_preserve_order_and_independent_clones() {
    let expected = expected_values("ttf");
    assert_eq!(expected.len(), 3, "fixture needs three MIME alternatives");
    let mut iter = from_ext("ttf").iter();
    assert_eq!(iter.next(), Some(expected[0].clone()));
    let mut cloned = iter.clone();
    assert_eq!(cloned.nth_back(1), Some(expected[1].clone()));
    assert_exhausted(&mut cloned);

    assert_eq!(iter.len(), 2);
    assert_eq!(iter.nth(0), Some(expected[1].clone()));
    assert_eq!(iter.clone().count(), 1);
    assert_eq!(iter.last(), Some(expected[2].clone()));

    let mut iter = from_ext("ttf").iter();
    assert_eq!(iter.next_back(), Some(expected[2].clone()));
    assert_eq!(iter.clone().count(), 2);
    assert_eq!(iter.nth(1), Some(expected[1].clone()));
    assert_exhausted(&mut iter);

    let mut iter = from_ext("ttf").iter();
    assert_eq!(iter.nth(1), Some(expected[1].clone()));
    assert_eq!(iter.nth_back(0), Some(expected[2].clone()));
    assert_exhausted(&mut iter);
}

#[test]
fn count_and_last_use_the_remaining_range_after_both_ends_advance() {
    for ext in ["", "unknown-extension", "txt", "ttf"] {
        let expected = expected_values(ext);
        for front in 0..=expected.len() {
            for back in 0..=expected.len() - front {
                let mut iter = from_ext(ext).iter();
                for value in &expected[..front] {
                    assert_eq!(iter.next(), Some(value.clone()));
                }
                for value in expected[expected.len() - back..].iter().rev() {
                    assert_eq!(iter.next_back(), Some(value.clone()));
                }
                let remaining = &expected[front..expected.len() - back];
                assert_eq!(iter.len(), remaining.len());
                assert_eq!(iter.size_hint(), (remaining.len(), Some(remaining.len())));
                assert_eq!(iter.clone().last(), remaining.last().cloned());
                assert_eq!(iter.count(), remaining.len());
            }
        }
    }
}

#[test]
fn nth_and_nth_back_match_slice_selection_and_exhaustion() {
    for ext in ["", "unknown-extension", "txt", "ttf"] {
        let expected = expected_values(ext);
        for n in (0..=expected.len() + 1).chain(std::iter::once(usize::MAX)) {
            let remaining = expected.len().saturating_sub(n.saturating_add(1));
            let mut iter = from_ext(ext).iter();
            assert_eq!(iter.nth(n), expected.get(n).cloned());
            assert_eq!(iter.len(), remaining);
            assert_eq!(
                iter.by_ref().collect::<Vec<_>>(),
                expected.iter().skip(n).skip(1).cloned().collect::<Vec<_>>()
            );
            assert_exhausted(&mut iter);

            let mut iter = from_ext(ext).iter();
            assert_eq!(iter.nth_back(n), expected.iter().rev().nth(n).cloned());
            assert_eq!(iter.len(), remaining);
            assert_eq!(iter.by_ref().collect::<Vec<_>>(), expected[..remaining]);
            assert_exhausted(&mut iter);
        }
    }
}
