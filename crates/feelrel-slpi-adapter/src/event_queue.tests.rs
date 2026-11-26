use super::*;

#[test]
fn reports_loss_before_returning_the_remaining_events() {
    let mut queue = EventQueue::default();
    for value in 0..=CAPACITY {
        queue.push(value);
    }
    assert!(queue.next().is_err());
    assert_eq!(queue.next().unwrap(), Some(1));
}
