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

/// 快速分解版本（可能不完整，但速度快）
/// 用于高负载情况下的快速响应
pub fn factorize_fast(mut n: u64) -> Vec<u64> {
    let mut factors = Vec::new();

    // 只检查小质数
    let small_primes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29];

    for &p in &small_primes {
        while n % p == 0 {
            factors.push(p);
            n /= p;
        }
        if p * p > n {
            break;
        }
    }
    // 如果还有剩余且不大，直接作为质数
    if n > 1 && n < 1000 {
        factors.push(n);
    } else if n > 1 {
        // 大数剩余部分，标记为需要进一步分解
        factors.push(n);
        factors.push(0); // 0作为标记，表示需要进一步分解
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

    #[test]
    fn test_factorize_fast() {
        // 快速分解可能不完整，但应该能处理小因子
        assert_eq!(factorize_fast(84), vec![2, 2, 3, 7]);
        assert_eq!(factorize_fast(100), vec![2, 2, 5, 5]);
    }
}