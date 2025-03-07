// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// Interface for the ERC20 token standard
interface IERC20 {
    function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);
    function transfer(address recipient, uint256 amount) external returns (bool);
}

// USDCAtomicSwap contract
contract USDCAtomicSwap {
    // USDC token contract address on Ethereum
    IERC20 public usdcToken;

    // Swap details
    struct Swap {
        address initiator;
        address participant;
        uint256 amount;
        bytes32 hashLock;
        uint256 timeLock;
        bytes secret;
        bool redeemed;
        bool refunded;
    }

    mapping(bytes32 => Swap) public swaps;

    event SwapInitiated(bytes32 indexed swapID, address indexed initiator, address participant, uint256 amount, bytes32 hashLock, uint256 timeLock);
    event Redeemed(bytes32 indexed swapID, bytes secret);
    event Refunded(bytes32 indexed swapID);

    constructor(address _usdcTokenAddress) {
        usdcToken = IERC20(_usdcTokenAddress);
    }

    // Initiator locks tokens in the contract
    function initiateSwap(
        bytes32 _swapID,
        address _participant,
        uint256 _amount,
        bytes32 _hashLock,
        uint256 _timeLock
    ) external {
        require(swaps[_swapID].initiator == address(0), "Swap already exists");

        // Transfer USDC from initiator to the contract
        require(usdcToken.transferFrom(msg.sender, address(this), _amount), "Token transfer failed");

        // Create new swap
        swaps[_swapID] = Swap({
            initiator: msg.sender,
            participant: _participant,
            amount: _amount,
            hashLock: _hashLock,
            timeLock: block.timestamp + _timeLock,
            secret: "",
            redeemed: false,
            refunded: false
        });

        emit SwapInitiated(_swapID, msg.sender, _participant, _amount, _hashLock, swaps[_swapID].timeLock);
    }

    // Participant redeems the tokens by providing the correct secret
    function redeem(bytes32 _swapID, bytes calldata _secret) external {
        Swap storage swap = swaps[_swapID];

        require(swap.participant == msg.sender, "Not authorized");
        require(!swap.redeemed, "Already redeemed");
        require(!swap.refunded, "Already refunded");
        require(block.timestamp < swap.timeLock, "Time lock expired");
        require(sha256(_secret) == swap.hashLock, "Invalid secret");

        swap.secret = _secret;
        swap.redeemed = true;

        // Transfer USDC to participant
        require(usdcToken.transfer(swap.participant, swap.amount), "Token transfer failed");

        emit Redeemed(_swapID, _secret);
    }

    // Initiator refunds the tokens after time lock expires
    function refund(bytes32 _swapID) external {
        Swap storage swap = swaps[_swapID];

        require(swap.initiator == msg.sender, "Not authorized");
        require(!swap.redeemed, "Already redeemed");
        require(!swap.refunded, "Already refunded");
        require(block.timestamp >= swap.timeLock, "Time lock not yet expired");

        swap.refunded = true;

        // Transfer USDC back to initiator
        require(usdcToken.transfer(swap.initiator, swap.amount), "Token transfer failed");

        emit Refunded(_swapID);
    }

    // Getter function to retrieve swap details
    function getSwap(bytes32 _swapID) external view returns (
        address initiator,
        address participant,
        uint256 amount,
        bytes32 hashLock,
        uint256 timeLock,
        bytes memory secret,
        bool redeemed,
        bool refunded
    ) {
        Swap storage swap = swaps[_swapID];
        return (
            swap.initiator,
            swap.participant,
            swap.amount,
            swap.hashLock,
            swap.timeLock,
            swap.secret,
            swap.redeemed,
            swap.refunded
        );
    }
}
