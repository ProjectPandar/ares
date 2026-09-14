// Verbatim minimal subset of OrcaSlicer Int128.hpp for the chain-probe.
#pragma once
#include <cstdint>
#include <cstdlib>
#include <cassert>
class Int128 {
public:
	static int sign_determinant_2x2_filtered(int64_t a11, int64_t a12, int64_t a21, int64_t a22)
	{
		int64_t a11s = (a11 + (1 << 31)) >> 32;
		int64_t a12s = (a12 + (1 << 31)) >> 32;
		int64_t a21s = (a21 + (1 << 31)) >> 32;
		int64_t a22s = (a22 + (1 << 31)) >> 32;
		int64_t det  = a11s * a22s - a12s * a21s;
		int64_t err  = ((std::abs(a11s) + std::abs(a12s) + std::abs(a21s) + std::abs(a22s)) << 1) + 1;
		return (std::abs(det) > err) ?
			((det > 0) ? 1 : -1) :
			sign_determinant_2x2(a11, a12, a21, a22);
	}
	static int sign_determinant_2x2(int64_t a11, int64_t a12, int64_t a21, int64_t a22)
	{
		__int128 det = (__int128)a11 * a22 - (__int128)a12 * a21;
		return (det > 0) - (det < 0);
	}
};
