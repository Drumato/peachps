/// 32bit wordの数をオクテット数に変換する
pub fn double_words_as_octets(dword: usize) -> usize {
    dword << 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn double_words_as_octets_test() {
        assert_eq!(4, double_words_as_octets(1));
    }
}
