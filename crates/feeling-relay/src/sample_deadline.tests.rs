use super::*;

#[test]
fn deadline_moves_only_when_a_sample_arrives() {
    let now = Instant::now();
    let interval = Duration::from_secs(4);
    let mut deadline = SampleDeadline::new(now, interval);
    assert_eq!(deadline.expires_at(), now + interval);
    deadline.record_sample(now + Duration::from_secs(2));
    assert_eq!(deadline.expires_at(), now + Duration::from_secs(6));
}
