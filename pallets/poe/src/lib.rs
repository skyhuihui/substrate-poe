#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{
	decl_module, decl_storage, decl_event, decl_error, ensure, StorageMap
};
use frame_system::ensure_signed;
use sp_std::vec::Vec;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Config: frame_system::Config {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	/// 我们只想要约束 trait 里边符合某些要求，但是不想限制他到底具体是什么类型的时候（交给实现这个 trait 的时候定义），我们会使用类似：
	// type event: From ……
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	trait Store for Module<T: Config> as TemplateModule {
        /// The storage item for our proofs.
        /// It maps a proof to the user who made the claim and when they made it.
        Proofs: map hasher(blake2_128_concat) Vec<u8> => (T::AccountId, T::BlockNumber);
    }
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Config>::AccountId {
        /// Event emitted when a proof has been claimed. [who, claim]
        ClaimCreated(AccountId, Vec<u8>),
        /// Event emitted when a claim is revoked by the owner. [who, claim]
        ClaimRevoked(AccountId, Vec<u8>),
        /// Event emitted when a claim is revoked by the owner. [receiver, claim]
        ClaimTransfer(AccountId, Vec<u8>),
    }
);

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Config> {
        /// The proof has already been claimed.
        ProofAlreadyClaimed,
        /// The proof does not exist, so it cannot be revoked.
        NoSuchProof,
        /// The proof is claimed by another account, so caller can't revoke it.
        NotProofOwner,
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        /// Allow a user to claim ownership of an unclaimed proof.
        #[weight = 10_000]
        fn create_claim(origin, proof: Vec<u8>) {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let sender = ensure_signed(origin)?;

            // Verify that the specified proof has not already been claimed.
            ensure!(!Proofs::<T>::contains_key(&proof), Error::<T>::ProofAlreadyClaimed);

            // Get the block number from the FRAME System module.
            let current_block = <frame_system::Module<T>>::block_number();

            // Store the proof with the sender and block number.
            Proofs::<T>::insert(&proof, (&sender, current_block));

            // Emit an event that the claim was created.
            Self::deposit_event(RawEvent::ClaimCreated(sender, proof));
        }

        /// Allow the owner to revoke their claim.
        #[weight = 10_000]
        fn revoke_claim(origin, proof: Vec<u8>) {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let sender = ensure_signed(origin)?;

            // Verify that the specified proof has been claimed.
            ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);

            // Get owner of the claim.
            let (owner, _) = Proofs::<T>::get(&proof);

            // Verify that sender of the current call is the claim owner.
            // 验证为真，则继续往下走
            ensure!(sender == owner, Error::<T>::NotProofOwner);

            // Remove claim from storage.
            Proofs::<T>::remove(&proof);

            // Emit an event that the claim was erased.
            Self::deposit_event(RawEvent::ClaimRevoked(sender, proof));
        }

        #[weight = 10_000]
        fn revoke_transfer(origin, proof: Vec<u8>) {
			// 版权的接收者来调用， origin中的签名者，变成了此次版权的所有人
			// 获得签名者
            let receiver  = ensure_signed(origin)?;

            // Verify that the specified proof has been claimed.
            // 校验指定的证明是否被声明
            ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);

      		let current_block = <frame_system::Module<T>>::block_number();
            // Remove claim from storage.
            Proofs::<T>::insert(&proof, (&receiver, current_block));

            // Emit an event that the claim was erased.
            Self::deposit_event(RawEvent::ClaimTransfer(receiver, proof));
        }
    }
}
