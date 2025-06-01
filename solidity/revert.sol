// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

uint constant depth = 4;

contract RevertFactory {
    address public root_;
    address public new_;
    uint public level_ = 0;

    function deploy() public {
        address old = new_;

        Revert x =  new Revert(level_);
        new_ = address(x);

        if (old != address(0)) {
            Revert(old).set_next(new_);
        } else {
            root_ = new_;
        }

        if (level_ < depth) {
            level_ = level_ + 1;
            deploy();
        }
    }

    function case_1() public {
        Revert(root_).case_1();
    }

    function case_2() public {
        Revert(root_).case_2();
    }

    function case_3() public {
        Revert(root_).case_3();
    }

    function case_4() public {
        Revert(root_).case_4();
    }
}

contract Revert {
    uint public level_;
    uint public value_;
    address public next_;

    constructor(uint lev) {
        level_ = lev;
    }

    function set_value(uint x) public {
        value_ = x;
    }

    function is_last() public view returns(bool) {
        return (next_ == address(0));
    }

    function next() public view returns (address) {
        return next_;
    }

    function set_next(address addr) public {
        next_ = addr;
    }

    function value() public view returns (uint256) {
        return value_;
    }

    function reset() public {
        value_ = 0;

        if (!is_last()) {
            Revert(next_).reset();
        }
    }

    function check_value_case1(uint revert_level) public {
        if (level_ >= revert_level) {
            require(value_ == 0);
        } else {
            require(value_ == 1);
        }

        if (!is_last()) {
            Revert(next_).check_value_case1(revert_level);
        }
    }

    function inc_value_case1(uint revert_level) public {
        value_ = value_ + 1;
        bool ok;

        if (!is_last()) {
            (ok,) = next_.call(abi.encodeWithSignature("inc_value_case1(uint256)", revert_level));
        }

        if (level_ == revert_level) {
            revert();
        }
    }

    function case_1() public {
        for (uint level=1; level<=depth; level++) {
            reset();
            inc_value_case1(level);
            check_value_case1(level);
        }
    }

    function set_value_case2(uint256 x, uint256 revert_level) public {
        value_ = x + 1;
        bool ok;

        if (!is_last()) {
            (ok,) = next_.call(abi.encodeWithSignature("set_value_case2(uint256,uint256)", value_, revert_level));
        }

        if (level_ == revert_level) {
            revert();
        }
    }

    function case_2() public {
        reset();
        set_value_case2(0, 2);

        Revert x0 = Revert(this);
        Revert x1 = Revert(x0.next());
        Revert x2 = Revert(x1.next());
        Revert x3 = Revert(x2.next());
        Revert x4 = Revert(x3.next());

        require(x0.value() == 1, "x0");
        require(x1.value() == 2, "x1");
        require(x2.value() == 0, "x2");
        require(x3.value() == 0, "x3");
        require(x4.value() == 0, "x4");
    }


    function set_value_case3(uint256 x, uint256 y, uint256 z, bool rev) public {
        value_ = x + 1;
        bool ok;

        if (!is_last()) {
            (ok,) = next_.call(abi.encodeWithSignature("set_value_case3(uint256,uint256,uint256,bool)", value_, y, z,rev));
        }

        if (level_ == y) {
            (ok,) = next_.call(abi.encodeWithSignature("set_value_case3(uint256,uint256,uint256,bool)", value_, y, z,!rev));
        }

        if (level_ == z && rev) {
            revert();
        }
    }

    function case_3() public {
        reset();
        set_value_case3(0, 3, 4, false);

        Revert x0 = Revert(this);
        Revert x1 = Revert(x0.next());
        Revert x2 = Revert(x1.next());
        Revert x3 = Revert(x2.next());
        Revert x4 = Revert(x3.next());

        require(x0.value() == 1, "x0");
        require(x1.value() == 2, "x1");
        require(x2.value() == 3, "x2");
        require(x3.value() == 4, "x3");
        require(x4.value() == 5, "x4");
    }

   function inc_value_case4(uint256 y, uint256 z, bool rev) public {
        value_ = value_ + 1;
        bool ok;

        if (!is_last()) {
            (ok,) = next_.call(abi.encodeWithSignature("inc_value_case4(uint256,uint256,bool)", y, z,rev));
        }

        if (level_ == y) {
            (ok,) = next_.call(abi.encodeWithSignature("inc_value_case4(uint256,uint256,bool)", y, z,!rev));
        }

        if (level_ == z && rev) {
            revert();
        }
    }

    function case_4() public {
        reset();

        Revert x0 = Revert(this);
        Revert x1 = Revert(x0.next());
        Revert x2 = Revert(x1.next());
        Revert x3 = Revert(x2.next());
        Revert x4 = Revert(x3.next());

        x0.set_value(0);
        x1.set_value(1);
        x2.set_value(2);
        x3.set_value(3);
        x4.set_value(4);

        inc_value_case4(2, 4, false);

        require(x0.value() == 1, "x0");
        require(x1.value() == 2, "x1");
        require(x2.value() == 3, "x2");
        require(x3.value() == 5, "x3");
        require(x4.value() == 5, "x4");
    }
}
