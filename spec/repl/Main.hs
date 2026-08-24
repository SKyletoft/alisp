module Main where

import MAlonzo.Code.Language (T_Expr_24(..), d_eval_794)
import qualified Data.Text as T
import System.IO (isEOF)
import Alisp.Abs
import Alisp.Par (pExpr, myLexer)
import Alisp.ErrM

convert :: Expr -> T_Expr_24
convert (EAtom (Ident s)) = C_atom_62 (T.pack s)
convert (ENumber n)       = C_number_64 n
convert (EString s)       = C_atom_62 (T.pack s)
convert (EList xs)        = listToPairs (map convert xs)
convert (EDotted a b)     = C_pair_66 (convert a) (convert b)
convert (EQuote e)        = C_quot_68 (convert e)
convert (EQuasi e)        = C_quasiquot_70 (convert e)
convert (EUnquote e)      = C_unquot_72 (convert e)

listToPairs :: [T_Expr_24] -> T_Expr_24
listToPairs []     = C_atom_62 (T.pack "nil")
listToPairs (x:xs) = C_pair_66 x (listToPairs xs)

instance Show T_Expr_24 where
  showsPrec p (C_atom_62 t)      = showParen (p > 10) $ showString "atom " . shows (T.unpack t)
  showsPrec p (C_number_64 n)    = showParen (p > 10) $ showString "number " . shows n
  showsPrec p (C_pair_66 a b)    = showParen (p > 10) $ showString "pair " . showsPrec 11 a . showString " " . showsPrec 11 b
  showsPrec p (C_quot_68 e)      = showParen (p > 10) $ showString "quot " . showsPrec 11 e
  showsPrec p (C_quasiquot_70 e) = showParen (p > 10) $ showString "quasiquot " . showsPrec 11 e
  showsPrec p (C_unquot_72 e)    = showParen (p > 10) $ showString "unquot " . showsPrec 11 e

fuel :: Integer
fuel = 1000

main :: IO ()
main = loop
  where
    loop = do
      eof <- isEOF
      if eof then pure () else do
        line <- getLine
        if all (`elem` " \t\r") line then loop else
          case pExpr (myLexer line) of
            Bad _ -> putStrLn "parse error" >> loop
            Ok e  -> let expr = convert e in
              case d_eval_794 fuel expr of
                Nothing -> putStrLn "Nothing" >> loop
                Just v  -> print e >> print expr >> print v >> loop
