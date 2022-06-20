# iter-sk(ip)(t)ak(e)

This creates a new iterator type that combines the functionality of std::iter::Skip and std::iter::Take.
Usage is as follows 
```rs
let v: Vec<i32> = vec![1,2,3,4,5,6,7,8];
// Takes the first 2 values of `v` into `taken` and makes the rest of the iterator accessible through `next`
let (mut taken, mut next) = Skak::new(v.iter(), 2);
let mut count = 0;
assert_eq!(next.size_hint().0, v.len() - 2);
while next.size_hint().0 > 0 {
	println!("Set {}", count);
	// You can then call `Skak::skip` with your previous iterator, and a new amount of elements to be skipped
	(taken, next) = Skak::skip(next, 2);
	count += 1;
}
```
