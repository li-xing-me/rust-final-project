pub fn factorize(mut n: u64) -> Vec<u64> {
    let mut factors = Vec::new();

    // 处理因子2
    while n % 2 == 0 {
        factors.push(2);
        n /= 2;
    }

    // 处理奇数因子
    let mut i = 3;
    while i * i <= n {
        while n % i == 0 {
            factors.push(i);
            n /= i;
        }
        i += 2;
    }

    // 如果剩余部分大于1，则它本身是质数
    if n > 1 {
        factors.push(n);
    }

    factors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factorize() {
        assert_eq!(factorize(2), vec![2]);
        assert_eq!(factorize(15), vec![3, 5]);
        assert_eq!(factorize(84), vec![2, 2, 3, 7]);
        assert_eq!(factorize(997), vec![997]); // 质数
    }
}