pragma solidity =0.5.16;

import './UniswapV2Pair.sol';
import './UniswapV2Pair_1.sol';
import './UniswapV2Pair_2.sol';
import './UniswapV2Pair_3.sol';
import './UniswapV2Pair_4.sol';
import './UniswapV2Pair_5.sol';
import './UniswapV2Pair_6.sol';
import './UniswapV2Pair_7.sol';
import './UniswapV2ERC20_1.sol';

contract Huge {

    function deploy() public{
        UniswapV2Pair pair = new UniswapV2Pair();
    }
    function deploy_1() public {
        UniswapV2Pair_1 pair_1 = new UniswapV2Pair_1();
    }
    function deploy_2() public {
        UniswapV2Pair_2 pair_2 = new UniswapV2Pair_2();
    }
    function deploy_3() public {
        UniswapV2Pair_3 pair_3 = new UniswapV2Pair_3();
    }
    function deploy_4() public {
        UniswapV2Pair_4 pair_4 = new UniswapV2Pair_4();
    }
    function deploy_5() public {
        UniswapV2Pair_5 pair_5 = new UniswapV2Pair_5();
    }
    function deploy_6() public {
        UniswapV2Pair_6 pair_6 = new UniswapV2Pair_6();
    }
    function deploy_7() public {
        UniswapV2Pair_7 pair_7 = new UniswapV2Pair_7();
    }
    function deploy_erc20_1() public{
        UniswapV2ERC20_1 erc_20 = new UniswapV2ERC20_1();
    }
}